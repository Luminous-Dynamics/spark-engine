//! Weighted closest-viable Q_econ design projection.
//!
//! This module implements the weighted least-squares endpoint problem from
//! Whyte et al. (2026), Eq. 52, over a strictly-positive box-constrained
//! parameter domain. It does not claim to generate an R&D trajectory.
//!
//! The objective is evaluated in the original physical/economic variables:
//!
//!     sum_i difficulty_i * ((theta_i - theta_reference_i) / scale_i)^2
//!
//! The log transform is used only as the numerical optimization coordinate.
//! Under the paper's strictly-positive domain and lower-bound condition
//! (lower_i > theta_reference_i / 2), the objective is convex in log
//! coordinates. The Q_econ super-level set is also convex in log coordinates
//! by the paper's log-log-concavity result. A logarithmic barrier therefore
//! gives a convex interior problem for each fixed (integer) plant lifetime.
//!
//! Plant lifetime is an integer in Spark's public Qecon API. Instead of
//! relaxing and rounding it, this solver enumerates each allowed integer year
//! and solves the continuous nine-dimensional subproblem for that year.
//!
//! Difficulty weights are stakeholder assumptions, not objective truths.
//! Changing them changes the selected closest point, never the Q_econ
//! viability boundary.

use crate::qecon::{QECON_MODEL_REVISION, QeconInput, calculate_qecon};
use crate::qecon_exploration::QeconParameter;
use serde::{Deserialize, Serialize};

pub const QECON_PROJECTION_REVISION: &str = "qecon-weighted-projection-v0.1";
pub const QECON_PROJECTION_MAX_LIFETIME_SPAN_YEARS: u32 = 250;

const BARRIER_PARAMETERS: [f64; 8] = [
    1e-1, 3e-2, 1e-2, 3e-3, 1e-3, 3e-4, 1e-4, 3e-5,
];
const MAX_COORDINATE_SWEEPS: usize = 80;
const GOLDEN_SECTION_STEPS: usize = 36;
const COORDINATE_TOLERANCE: f64 = 2e-8;
const FEASIBILITY_EPSILON: f64 = 1e-12;

/// Positive box bounds, scale and stakeholder difficulty for one parameter.
///
/// The combined diagonal objective weight is difficulty_weight / scale².
/// Bounds and scale use the parameter's native units; difficulty_weight is
/// dimensionless and strictly positive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconProjectionBound {
    pub parameter: QeconParameter,
    pub minimum: f64,
    pub maximum: f64,
    pub scale: f64,
    pub difficulty_weight: f64,
}

/// Input for a weighted closest-viable-design projection.
///
/// Exactly one bound must be supplied for every one of the ten Q_econ
/// parameters. The reference design may lie outside the box; it is projected
/// against the declared bounded design domain. A reference value of zero is
/// permitted when the positive optimization lower bound satisfies the
/// convexity condition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconWeightedProjectionInput {
    pub reference: QeconInput,
    pub target_q_econ: f64,
    pub bounds: Vec<QeconProjectionBound>,
}

/// Whether a parameter is fixed at a bound or lies in the box interior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconProjectionBoundState {
    Fixed,
    Lower,
    Interior,
    Upper,
}

/// Convergence interpretation for the endpoint projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconProjectionStatus {
    /// The reference design itself is in the box and already reaches target.
    ReferenceAlreadyViable,
    /// The weighted barrier/coordinate projection converged.
    Converged,
    /// Only the favourable box corner reaches the target (within tolerance).
    BoundaryOnly,
    /// The feasible endpoint was found, but the configured iteration budget
    /// was exhausted before the coordinate convergence threshold was met.
    IterationLimit,
}

/// Per-parameter displacement and its contribution to the weighted objective.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconProjectionParameterResult {
    pub parameter: QeconParameter,
    pub unit: String,
    pub reference_value: f64,
    pub projected_value: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub scale: f64,
    pub difficulty_weight: f64,
    pub weighted_distance_contribution: f64,
    pub bound_state: QeconProjectionBoundState,
}

/// Auditable weighted closest-viable Q_econ endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconWeightedProjectionResult {
    pub model_revision: String,
    pub projection_revision: String,
    pub target_q_econ: f64,
    pub projected_input: QeconInput,
    pub projected_q_econ: f64,
    pub weighted_distance_squared: f64,
    pub kkt_stationarity_residual: f64,
    pub status: QeconProjectionStatus,
    pub integer_lifetime_candidates_examined: u32,
    /// Coordinate sweeps used for the selected lifetime candidate.
    pub coordinate_sweeps: u64,
    /// Total coordinate sweeps across all lifetime candidates evaluated.
    pub total_coordinate_sweeps: u64,
    pub final_barrier_parameter: f64,
    pub parameter_results: Vec<QeconProjectionParameterResult>,
    pub assumptions: Vec<String>,
}

/// Find the minimum weighted-distance design in the supplied box satisfying
/// Q_econ >= target_q_econ.
///
/// The optimization is a logarithmic-barrier path in log(theta) coordinates.
/// For each permitted integer plant lifetime, the nine continuous parameters
/// are optimized by cyclic bounded one-dimensional golden-section minimization
/// (an approximate coordinate minimizer) inside the feasible interval.
/// The best feasible endpoint across all integer lifetimes is returned.
///
/// The paper's convexity/uniqueness result applies only on the strictly
/// positive box with lower_i > reference_i / 2. This function enforces that
/// condition and reports iteration-limit status rather than silently claiming
/// global convergence when coordinate sweeps do not meet tolerance.
pub fn project_qecon_to_viability(
    request: &QeconWeightedProjectionInput,
) -> Result<QeconWeightedProjectionResult, String> {
    validate_projection_input(request)?;
    let ordered_bounds = order_projection_bounds(&request.bounds)?;
    let reference_values = input_values(&request.reference);

    if reference_is_inside_box(&reference_values, &ordered_bounds) {
        let reference_result = calculate_qecon(&request.reference)?;
        if reference_result.q_econ >= request.target_q_econ {
            return build_projection_result(
                request,
                &ordered_bounds,
                request.reference.clone(),
                QeconProjectionStatus::ReferenceAlreadyViable,
                0,
                0,
                0,
                0.0,
            );
        }
    }

    let lifetime_bound = &ordered_bounds[QeconParameter::PlantLifetime.index()];
    let first_lifetime = lifetime_bound.minimum as u32;
    let last_lifetime = lifetime_bound.maximum as u32;
    let lifetime_candidates = last_lifetime - first_lifetime + 1;

    let mut best: Option<CandidateProjection> = None;
    let mut total_sweeps = 0_u64;
    let mut candidates_examined = 0_u32;

    for lifetime in first_lifetime..=last_lifetime {
        candidates_examined += 1;
        let mut corner_values = favourable_corner(&ordered_bounds);
        corner_values[QeconParameter::PlantLifetime.index()] = f64::from(lifetime);
        let corner_input = input_from_values(&request.reference, &corner_values)?;
        let corner_q = calculate_qecon(&corner_input)?.q_econ;
        if corner_q + FEASIBILITY_EPSILON < request.target_q_econ {
            continue;
        }

        let mut clamped_reference = clamp_reference(&reference_values, &ordered_bounds);
        clamped_reference[QeconParameter::PlantLifetime.index()] = f64::from(lifetime);
        let clamped_input = input_from_values(&request.reference, &clamped_reference)?;
        let clamped_q = calculate_qecon(&clamped_input)?.q_econ;

        let candidate = if clamped_q >= request.target_q_econ {
            CandidateProjection {
                input: clamped_input,
                status: QeconProjectionStatus::Converged,
                coordinate_sweeps: 0,
                final_barrier_parameter: 0.0,
            }
        } else if corner_q <= request.target_q_econ * (1.0 + 1e-12) {
            CandidateProjection {
                input: corner_input,
                status: QeconProjectionStatus::BoundaryOnly,
                coordinate_sweeps: 0,
                final_barrier_parameter: 0.0,
            }
        } else {
            let start_values = strict_feasible_start(
                &request.reference,
                &corner_values,
                &clamped_reference,
                request.target_q_econ,
            )?;
            let (projected_values, sweeps, converged, barrier) = optimize_fixed_lifetime(
                &request.reference,
                &ordered_bounds,
                &start_values,
                request.target_q_econ,
            )?;
            total_sweeps += sweeps as u64;
            let projected_input = input_from_values(&request.reference, &projected_values)?;
            let projected_q = calculate_qecon(&projected_input)?.q_econ;
            if projected_q + FEASIBILITY_EPSILON < request.target_q_econ {
                return Err(format!(
                    "projection failed to maintain feasibility for lifetime {lifetime}: Q_econ={projected_q}"
                ));
            }
            CandidateProjection {
                input: projected_input,
                status: if converged {
                    QeconProjectionStatus::Converged
                } else {
                    QeconProjectionStatus::IterationLimit
                },
                coordinate_sweeps: sweeps,
                final_barrier_parameter: barrier,
            }
        };

        let distance = weighted_distance_squared(
            &candidate.input,
            &request.reference,
            &ordered_bounds,
        );
        let replace = best.as_ref().is_none_or(|current| {
            distance < weighted_distance_squared(
                &current.input,
                &request.reference,
                &ordered_bounds,
            )
        });
        if replace {
            best = Some(candidate);
        }
    }

    let best = best.ok_or_else(|| {
        "no Q_econ-viable design exists inside the declared positive parameter box and integer-lifetime range".to_string()
    })?;

    build_projection_result(
        request,
        &ordered_bounds,
        best.input,
        best.status,
        candidates_examined.min(lifetime_candidates),
        best.coordinate_sweeps as u64,
        total_sweeps,
        best.final_barrier_parameter,
    )
}

#[derive(Debug, Clone)]
struct CandidateProjection {
    input: QeconInput,
    status: QeconProjectionStatus,
    coordinate_sweeps: usize,
    final_barrier_parameter: f64,
}

pub(crate) fn validate_projection_input(
    request: &QeconWeightedProjectionInput,
) -> Result<(), String> {
    calculate_qecon(&request.reference)?;
    if !request.target_q_econ.is_finite() || request.target_q_econ <= 0.0 {
        return Err("target_q_econ must be finite and positive".to_string());
    }
    if request.bounds.len() != QeconParameter::ALL.len() {
        return Err("exactly ten Q_econ parameter bounds are required".to_string());
    }

    let mut seen = [false; 10];
    for bound in &request.bounds {
        let index = bound.parameter.index();
        if seen[index] {
            return Err(format!("duplicate projection bound for {:?}", bound.parameter));
        }
        seen[index] = true;

        if !bound.minimum.is_finite()
            || !bound.maximum.is_finite()
            || bound.minimum <= 0.0
            || bound.maximum < bound.minimum
        {
            return Err(format!(
                "bounds for {:?} must satisfy finite 0 < minimum <= maximum",
                bound.parameter
            ));
        }
        if !bound.scale.is_finite() || bound.scale <= 0.0 {
            return Err(format!("scale for {:?} must be finite and positive", bound.parameter));
        }
        if !bound.difficulty_weight.is_finite() || bound.difficulty_weight <= 0.0 {
            return Err(format!(
                "difficulty_weight for {:?} must be finite and positive",
                bound.parameter
            ));
        }
        let reference_value = bound.parameter.value(&request.reference);
        if !reference_value.is_finite() {
            return Err(format!("reference value for {:?} must be finite", bound.parameter));
        }
        if bound.minimum <= reference_value / 2.0 {
            return Err(format!(
                "lower bound for {:?} must be strictly greater than half the reference value for the paper's log-coordinate uniqueness condition",
                bound.parameter
            ));
        }
        bound
            .parameter
            .with_value(&request.reference, bound.minimum)?;
        bound
            .parameter
            .with_value(&request.reference, bound.maximum)?;

        if bound.parameter == QeconParameter::PlantLifetime {
            if bound.minimum.fract() != 0.0
                || bound.maximum.fract() != 0.0
                || bound.minimum < 1.0
                || bound.maximum > f64::from(u32::MAX)
            {
                return Err(
                    "plant lifetime bounds must be positive integer years within u32 range"
                        .to_string(),
                );
            }
            let span = bound.maximum - bound.minimum;
            if span > f64::from(QECON_PROJECTION_MAX_LIFETIME_SPAN_YEARS) {
                return Err(format!(
                    "plant lifetime search span exceeds {QECON_PROJECTION_MAX_LIFETIME_SPAN_YEARS} years"
                ));
            }
        }
    }
    if seen.iter().any(|present| !present) {
        return Err("all ten Q_econ parameters need one projection bound".to_string());
    }
    let ordered = order_projection_bounds(&request.bounds)?;
    for parameter in QeconParameter::ALL {
        let bound = &ordered[parameter.index()];
        if parameter != QeconParameter::PlantLifetime
            && bound.minimum <= 0.0
        {
            return Err("log-coordinate projection requires strictly positive bounds".to_string());
        }
    }
    let lifetime = &ordered[QeconParameter::PlantLifetime.index()];
    if lifetime.minimum > lifetime.maximum {
        return Err("integer plant lifetime range is empty".to_string());
    }
    Ok(())
}

pub(crate) fn order_projection_bounds(
    bounds: &[QeconProjectionBound],
) -> Result<[QeconProjectionBound; 10], String> {
    let mut ordered: [Option<QeconProjectionBound>; 10] =
        std::array::from_fn(|_| None);
    for bound in bounds {
        ordered[bound.parameter.index()] = Some(bound.clone());
    }
    let mut values = Vec::with_capacity(10);
    for bound in ordered {
        values.push(bound.ok_or_else(|| "missing Q_econ projection bound".to_string())?);
    }
    values
        .try_into()
        .map_err(|_| "expected ten Q_econ projection bounds".to_string())
}

pub(crate) fn input_values(input: &QeconInput) -> [f64; 10] {
    std::array::from_fn(|index| QeconParameter::ALL[index].value(input))
}

pub(crate) fn input_from_values(reference: &QeconInput, values: &[f64; 10]) -> Result<QeconInput, String> {
    let mut input = reference.clone();
    for parameter in QeconParameter::ALL {
        input = parameter.with_value(&input, values[parameter.index()])?;
    }
    Ok(input)
}

fn reference_is_inside_box(
    reference: &[f64; 10],
    bounds: &[QeconProjectionBound; 10],
) -> bool {
    QeconParameter::ALL.into_iter().all(|parameter| {
        let index = parameter.index();
        reference[index] >= bounds[index].minimum
            && reference[index] <= bounds[index].maximum
    })
}

pub(crate) fn clamp_reference(
    reference: &[f64; 10],
    bounds: &[QeconProjectionBound; 10],
) -> [f64; 10] {
    std::array::from_fn(|index| {
        reference[index].clamp(bounds[index].minimum, bounds[index].maximum)
    })
}

pub(crate) fn favourable_corner(bounds: &[QeconProjectionBound; 10]) -> [f64; 10] {
    std::array::from_fn(|index| {
        let bound = &bounds[index];
        match bound.parameter {
            QeconParameter::FusionPowerDensity
            | QeconParameter::SurfaceEnergyFluenceLimit
            | QeconParameter::NetEnergyConversionEfficiency
            | QeconParameter::NetPriceOfEnergy
            | QeconParameter::PlantLifetime => bound.maximum,
            QeconParameter::SurfaceReplacementTime
            | QeconParameter::TargetCostPerEnergyYield
            | QeconParameter::SurfaceReplacementCost
            | QeconParameter::PlantArealCost
            | QeconParameter::RealInterestRate => bound.minimum,
        }
    })
}

fn q_at_values(reference: &QeconInput, values: &[f64; 10]) -> Result<f64, String> {
    let input = input_from_values(reference, values)?;
    Ok(calculate_qecon(&input)?.q_econ)
}

fn strict_feasible_start(
    reference: &QeconInput,
    corner: &[f64; 10],
    clamped_reference: &[f64; 10],
    target: f64,
) -> Result<[f64; 10], String> {
    let corner_q = q_at_values(reference, corner)?;
    if corner_q <= target {
        return Err("strict interior feasible point does not exist inside this lifetime slice".to_string());
    }
    let mut feasible_alpha = 0.0;
    let mut infeasible_alpha = 1.0;
    for _ in 0..64 {
        let alpha = (feasible_alpha + infeasible_alpha) * 0.5;
        let values = interpolate(corner, clamped_reference, alpha);
        if q_at_values(reference, &values)? > target {
            feasible_alpha = alpha;
        } else {
            infeasible_alpha = alpha;
        }
    }
    let alpha = feasible_alpha * 0.98;
    let start = interpolate(corner, clamped_reference, alpha);
    if q_at_values(reference, &start)? <= target {
        return Err("could not construct a strictly feasible projection start".to_string());
    }
    Ok(start)
}

fn interpolate(left: &[f64; 10], right: &[f64; 10], alpha: f64) -> [f64; 10] {
    std::array::from_fn(|index| left[index] + alpha * (right[index] - left[index]))
}

pub(crate) fn weighted_distance_squared(
    input: &QeconInput,
    reference: &QeconInput,
    bounds: &[QeconProjectionBound; 10],
) -> f64 {
    QeconParameter::ALL
        .into_iter()
        .map(|parameter| {
            let index = parameter.index();
            let difference = parameter.value(input) - parameter.value(reference);
            bounds[index].difficulty_weight
                * (difference / bounds[index].scale).powi(2)
        })
        .sum()
}

fn barrier_value(
    reference: &QeconInput,
    values: &[f64; 10],
    bounds: &[QeconProjectionBound; 10],
    target: f64,
    barrier: f64,
) -> f64 {
    let Ok(input) = input_from_values(reference, values) else {
        return f64::INFINITY;
    };
    let Ok(result) = calculate_qecon(&input) else {
        return f64::INFINITY;
    };
    let margin = result.q_econ.ln() - target.ln();
    if !margin.is_finite() || margin <= 0.0 {
        return f64::INFINITY;
    }
    weighted_distance_squared(&input, reference, bounds) - barrier * margin.ln()
}

fn optimize_fixed_lifetime(
    reference: &QeconInput,
    bounds: &[QeconProjectionBound; 10],
    initial: &[f64; 10],
    target: f64,
) -> Result<([f64; 10], usize, bool, f64), String> {
    let mut values = *initial;
    let mut total_sweeps = 0;
    let lifetime_index = QeconParameter::PlantLifetime.index();
    let mut all_stages_converged = true;
    let mut final_barrier = 0.0;

    for barrier in BARRIER_PARAMETERS {
        final_barrier = barrier;
        let mut stage_converged = false;
        for _ in 0..MAX_COORDINATE_SWEEPS {
            total_sweeps += 1;
            let mut maximum_log_change: f64 = 0.0;
            for parameter in QeconParameter::ALL {
                let index = parameter.index();
                if index == lifetime_index {
                    continue;
                }
                let old_value = values[index];
                let (left, right) = feasible_coordinate_interval(
                    reference,
                    &values,
                    index,
                    bounds[index].minimum,
                    bounds[index].maximum,
                    target,
                )?;
                if right - left <= 1e-12 * left.abs().max(1.0) {
                    continue;
                }
                let log_left = left.ln();
                let log_right = right.ln();
                let optimum_log = golden_section_coordinate(
                    reference,
                    &values,
                    bounds,
                    index,
                    log_left,
                    log_right,
                    target,
                    barrier,
                );
                let candidate = optimum_log.exp();
                let mut candidate_values = values;
                candidate_values[index] =
                    candidate.clamp(bounds[index].minimum, bounds[index].maximum);
                if q_at_values(reference, &candidate_values)? >= target {
                    values = candidate_values;
                }
                maximum_log_change = maximum_log_change
                    .max((values[index].ln() - old_value.ln()).abs());
            }
            if maximum_log_change <= COORDINATE_TOLERANCE {
                stage_converged = true;
                break;
            }
        }
        all_stages_converged &= stage_converged;
    }
    Ok((values, total_sweeps, all_stages_converged, final_barrier))
}

fn feasible_coordinate_interval(
    reference: &QeconInput,
    current: &[f64; 10],
    coordinate: usize,
    minimum: f64,
    maximum: f64,
    target: f64,
) -> Result<(f64, f64), String> {
    let current_value = current[coordinate];
    if q_at_values(reference, current)? < target {
        return Err("coordinate minimization requires a feasible current point".to_string());
    }
    let mut left = minimum;
    let mut left_values = *current;
    left_values[coordinate] = minimum;
    if q_at_values(reference, &left_values)? < target {
        let mut infeasible = minimum;
        let mut feasible = current_value;
        for _ in 0..56 {
            let midpoint = (infeasible + feasible) * 0.5;
            let mut trial = *current;
            trial[coordinate] = midpoint;
            if q_at_values(reference, &trial)? >= target {
                feasible = midpoint;
            } else {
                infeasible = midpoint;
            }
        }
        left = feasible;
    }

    let mut right = maximum;
    let mut right_values = *current;
    right_values[coordinate] = maximum;
    if q_at_values(reference, &right_values)? < target {
        let mut feasible = current_value;
        let mut infeasible = maximum;
        for _ in 0..56 {
            let midpoint = (feasible + infeasible) * 0.5;
            let mut trial = *current;
            trial[coordinate] = midpoint;
            if q_at_values(reference, &trial)? >= target {
                feasible = midpoint;
            } else {
                infeasible = midpoint;
            }
        }
        right = feasible;
    }

    // Stay just inside the feasible interval so the logarithmic barrier is
    // finite at both ends.
    left += (current_value - left).max(0.0) * 1e-10;
    right -= (right - current_value).max(0.0) * 1e-10;
    Ok((left.min(current_value), right.max(current_value)))
}

#[allow(clippy::too_many_arguments)]
fn golden_section_coordinate(
    reference: &QeconInput,
    current: &[f64; 10],
    bounds: &[QeconProjectionBound; 10],
    coordinate: usize,
    mut left: f64,
    mut right: f64,
    target: f64,
    barrier: f64,
) -> f64 {
    const GOLDEN_RATIO_CONJUGATE: f64 = 0.618_033_988_749_894_9;
    let mut x1 = right - GOLDEN_RATIO_CONJUGATE * (right - left);
    let mut x2 = left + GOLDEN_RATIO_CONJUGATE * (right - left);
    let mut f1 = coordinate_barrier_value(
        reference, current, bounds, coordinate, x1, target, barrier,
    );
    let mut f2 = coordinate_barrier_value(
        reference, current, bounds, coordinate, x2, target, barrier,
    );

    for _ in 0..GOLDEN_SECTION_STEPS {
        if f1 <= f2 {
            right = x2;
            x2 = x1;
            f2 = f1;
            x1 = right - GOLDEN_RATIO_CONJUGATE * (right - left);
            f1 = coordinate_barrier_value(
                reference, current, bounds, coordinate, x1, target, barrier,
            );
        } else {
            left = x1;
            x1 = x2;
            f1 = f2;
            x2 = left + GOLDEN_RATIO_CONJUGATE * (right - left);
            f2 = coordinate_barrier_value(
                reference, current, bounds, coordinate, x2, target, barrier,
            );
        }
    }
    if f1 <= f2 { x1 } else { x2 }
}

#[allow(clippy::too_many_arguments)]
fn coordinate_barrier_value(
    reference: &QeconInput,
    current: &[f64; 10],
    bounds: &[QeconProjectionBound; 10],
    coordinate: usize,
    log_value: f64,
    target: f64,
    barrier: f64,
) -> f64 {
    let mut values = *current;
    values[coordinate] = log_value.exp();
    barrier_value(reference, &values, bounds, target, barrier)
}

pub(crate) fn bound_state(value: f64, bound: &QeconProjectionBound) -> QeconProjectionBoundState {
    if bound.minimum == bound.maximum {
        QeconProjectionBoundState::Fixed
    } else if (value - bound.minimum).abs() <= 1e-7 * bound.scale.max(1.0) {
        QeconProjectionBoundState::Lower
    } else if (bound.maximum - value).abs() <= 1e-7 * bound.scale.max(1.0) {
        QeconProjectionBoundState::Upper
    } else {
        QeconProjectionBoundState::Interior
    }
}

fn kkt_stationarity_residual(
    projected: &QeconInput,
    reference: &QeconInput,
    bounds: &[QeconProjectionBound; 10],
    target: f64,
) -> Result<f64, String> {
    let values = input_values(projected);
    let mut objective_gradient = [0.0; 10];
    let mut constraint_gradient = [0.0; 10];
    let mut states = [QeconProjectionBoundState::Interior; 10];

    for parameter in QeconParameter::ALL {
        let index = parameter.index();
        let bound = &bounds[index];
        if parameter == QeconParameter::PlantLifetime {
            states[index] = bound_state(values[index], bound);
            continue;
        }
        states[index] = bound_state(values[index], bound);
        let reference_value = parameter.value(reference);
        objective_gradient[index] = 2.0
            * bound.difficulty_weight
            * (values[index] - reference_value)
            / (bound.scale * bound.scale);

        let step = 1e-5 * values[index].abs().max(bound.scale * 1e-6).max(1e-8);
        let plus_value = (values[index] + step).min(bound.maximum);
        let minus_value = (values[index] - step).max(bound.minimum);
        if plus_value <= minus_value {
            continue;
        }
        let mut plus_values = values;
        let mut minus_values = values;
        plus_values[index] = plus_value;
        minus_values[index] = minus_value;
        let q_plus = q_at_values(reference, &plus_values)?;
        let q_minus = q_at_values(reference, &minus_values)?;
        constraint_gradient[index] =
            (q_plus.ln() - q_minus.ln()) / (plus_value - minus_value);
    }

    // Estimate the Lagrange multiplier on free coordinates in standardized
    // coordinates; active bounds are tested with one-sided KKT signs.
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for parameter in QeconParameter::ALL {
        let index = parameter.index();
        if parameter == QeconParameter::PlantLifetime
            || states[index] != QeconProjectionBoundState::Interior
        {
            continue;
        }
        let factor = bounds[index].scale / bounds[index].difficulty_weight.sqrt();
        let df = objective_gradient[index] * factor;
        let dg = constraint_gradient[index] * factor;
        numerator += df * dg;
        denominator += dg * dg;
    }
    let lambda = if denominator > 0.0 {
        (numerator / denominator).max(0.0)
    } else {
        0.0
    };

    let mut maximum_residual: f64 = 0.0;
    for parameter in QeconParameter::ALL {
        let index = parameter.index();
        if parameter == QeconParameter::PlantLifetime
            || states[index] == QeconProjectionBoundState::Fixed
        {
            continue;
        }
        let factor = bounds[index].scale / bounds[index].difficulty_weight.sqrt();
        let residual =
            (objective_gradient[index] - lambda * constraint_gradient[index]) * factor;
        let violation = match states[index] {
            QeconProjectionBoundState::Lower => (-residual).max(0.0),
            QeconProjectionBoundState::Upper => residual.max(0.0),
            QeconProjectionBoundState::Interior => residual.abs(),
            QeconProjectionBoundState::Fixed => 0.0,
        };
        maximum_residual = maximum_residual.max(violation);
    }
    Ok(maximum_residual)
}

#[allow(clippy::too_many_arguments)]
fn build_projection_result(
    request: &QeconWeightedProjectionInput,
    bounds: &[QeconProjectionBound; 10],
    projected_input: QeconInput,
    status: QeconProjectionStatus,
    lifetime_candidates_examined: u32,
    coordinate_sweeps: u64,
    total_coordinate_sweeps: u64,
    final_barrier_parameter: f64,
) -> Result<QeconWeightedProjectionResult, String> {
    let projected = calculate_qecon(&projected_input)?;
    if projected.q_econ + FEASIBILITY_EPSILON < request.target_q_econ {
        return Err("projection result does not satisfy the requested Q_econ threshold".to_string());
    }
    let distance = weighted_distance_squared(&projected_input, &request.reference, bounds);
    let mut parameter_results = Vec::with_capacity(10);
    for parameter in QeconParameter::ALL {
        let index = parameter.index();
        let bound = &bounds[index];
        let reference_value = parameter.value(&request.reference);
        let projected_value = parameter.value(&projected_input);
        let contribution = bound.difficulty_weight
            * ((projected_value - reference_value) / bound.scale).powi(2);
        parameter_results.push(QeconProjectionParameterResult {
            parameter,
            unit: parameter.unit().to_string(),
            reference_value,
            projected_value,
            minimum: bound.minimum,
            maximum: bound.maximum,
            scale: bound.scale,
            difficulty_weight: bound.difficulty_weight,
            weighted_distance_contribution: contribution,
            bound_state: bound_state(projected_value, bound),
        });
    }
    let kkt_residual = kkt_stationarity_residual(
        &projected_input,
        &request.reference,
        bounds,
        request.target_q_econ,
    )?;

    Ok(QeconWeightedProjectionResult {
        model_revision: QECON_MODEL_REVISION.to_string(),
        projection_revision: QECON_PROJECTION_REVISION.to_string(),
        target_q_econ: request.target_q_econ,
        projected_input,
        projected_q_econ: projected.q_econ,
        weighted_distance_squared: distance,
        kkt_stationarity_residual: kkt_residual,
        status,
        integer_lifetime_candidates_examined,
        coordinate_sweeps,
        total_coordinate_sweeps,
        final_barrier_parameter,
        parameter_results,
        assumptions: vec![
            "The objective is the paper's dimensionless weighted Euclidean distance in the original ten physical/economic parameters.".to_string(),
            "The optimization domain is strictly positive and box-constrained.".to_string(),
            "The lower-bound condition lower_i > reference_i / 2 is enforced for the paper's log-coordinate convexity/uniqueness result.".to_string(),
            "The numerical method uses a logarithmic-barrier path and cyclic coordinate minimization in log coordinates.".to_string(),
            "Plant lifetime is enumerated over permitted integer years; it is not relaxed and rounded.".to_string(),
            "Difficulty weights and scales are caller-supplied stakeholder assumptions, not measured universal intervention costs.".to_string(),
            "The result is an endpoint projection, not a literal R&D trajectory and not a physical feasibility proof.".to_string(),
            "The KKT residual is a finite-difference diagnostic; independent oracle comparison remains necessary before a globally optimal claim.".to_string(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qecon::QeconDesignParameters;

    fn reference_input(power_density: f64) -> QeconInput {
        QeconInput {
            fusion_power_density_mw_per_m2: power_density,
            design: QeconDesignParameters {
                surface_energy_fluence_limit_mw_year_per_m2: 3.125,
                surface_replacement_time_years: 0.1,
                net_energy_conversion_efficiency: 0.4,
                net_price_of_energy_usd_per_mwh: 100.0,
                target_cost_per_energy_yield_usd_per_mj: 0.0,
                surface_replacement_cost_musd_per_m2: 0.3,
                plant_areal_cost_musd_per_m2: 10.0,
                real_interest_rate_percent: 2.0,
                plant_lifetime_years: 30,
            },
        }
    }

    fn projection_bounds(
        reference: &QeconInput,
        variable: Option<QeconParameter>,
        alternative: Option<QeconParameter>,
        difficulty_first: f64,
        difficulty_second: f64,
    ) -> Vec<QeconProjectionBound> {
        QeconParameter::ALL
            .into_iter()
            .map(|parameter| {
                let value = parameter.value(reference);
                let (minimum, maximum, scale, difficulty_weight) =
                    if Some(parameter) == variable {
                        let (minimum, maximum) = match parameter {
                            QeconParameter::FusionPowerDensity => (value * 0.55, 6.0, 3.5),
                            QeconParameter::NetPriceOfEnergy => (value * 0.7, 250.0, 180.0),
                            _ => (value * 0.7, value * 1.5, value.max(1e-3)),
                        };
                        (minimum, maximum, scale, difficulty_first)
                    } else if Some(parameter) == alternative {
                        let (minimum, maximum, scale) = match parameter {
                            QeconParameter::FusionPowerDensity => (value * 0.55, 6.0, 3.5),
                            QeconParameter::NetPriceOfEnergy => (value * 0.7, 250.0, 180.0),
                            _ => (value * 0.7, value * 1.5, value.max(1e-3)),
                        };
                        (minimum, maximum, scale, difficulty_second)
                    } else if value == 0.0 {
                        (1e-9, 1e-6, 1e-6, 1.0)
                    } else {
                        (value, value, value.max(1e-3), 1.0)
                    };
                QeconProjectionBound {
                    parameter,
                    minimum,
                    maximum,
                    scale,
                    difficulty_weight,
                }
            })
            .collect()
    }

    fn request(reference: QeconInput, target: f64) -> QeconWeightedProjectionInput {
        QeconWeightedProjectionInput {
            bounds: projection_bounds(&reference, None, None, 1.0, 1.0),
            reference,
            target_q_econ: target,
        }
    }

    #[test]
    fn already_viable_reference_is_returned_with_zero_distance() {
        let reference = reference_input(3.7);
        let bounds = projection_bounds(&reference, None, None, 1.0, 1.0);
        let result = project_qecon_to_viability(&QeconWeightedProjectionInput {
            reference: reference.clone(),
            target_q_econ: 1.5,
            bounds,
        })
        .unwrap();
        assert_eq!(result.status, QeconProjectionStatus::ReferenceAlreadyViable);
        assert_eq!(result.weighted_distance_squared, 0.0);
        assert_eq!(result.projected_input, reference);
    }

    #[test]
    fn single_variable_projection_matches_analytic_required_density() {
        let reference = reference_input(2.5);
        let mut request = request(reference.clone(), 1.5);
        request.bounds = projection_bounds(
            &reference,
            Some(QeconParameter::FusionPowerDensity),
            None,
            1.0,
            1.0,
        );
        let expected = crate::qecon::required_fusion_power_density_for_qecon(
            &reference.design,
            request.target_q_econ,
        )
        .unwrap();
        let result = project_qecon_to_viability(&request).unwrap();
        let actual = result.projected_input.fusion_power_density_mw_per_m2;
        assert!((actual - expected).abs() / expected < 2e-3);
        assert!(result.projected_q_econ >= request.target_q_econ);
        assert!(result.kkt_stationarity_residual < 5e-2);
    }

    #[test]
    fn stakeholder_weights_change_the_selected_endpoint_not_the_viability_rule() {
        let reference = reference_input(2.5);
        let density_is_cheap = QeconWeightedProjectionInput {
            reference: reference.clone(),
            target_q_econ: 1.5,
            bounds: projection_bounds(
                &reference,
                Some(QeconParameter::FusionPowerDensity),
                Some(QeconParameter::NetPriceOfEnergy),
                0.05,
                20.0,
            ),
        };
        let mut price_is_cheap = density_is_cheap.clone();
        for bound in &mut price_is_cheap.bounds {
            if bound.parameter == QeconParameter::FusionPowerDensity {
                bound.difficulty_weight = 20.0;
            } else if bound.parameter == QeconParameter::NetPriceOfEnergy {
                bound.difficulty_weight = 0.05;
            }
        }
        let first = project_qecon_to_viability(&density_is_cheap).unwrap();
        let second = project_qecon_to_viability(&price_is_cheap).unwrap();
        assert!(first.projected_input.fusion_power_density_mw_per_m2
            > second.projected_input.fusion_power_density_mw_per_m2);
        assert!(first.projected_input.design.net_price_of_energy_usd_per_mwh
            < second.projected_input.design.net_price_of_energy_usd_per_mwh);
        assert!(first.projected_q_econ >= density_is_cheap.target_q_econ);
        assert!(second.projected_q_econ >= price_is_cheap.target_q_econ);
    }

    #[test]
    fn multiple_feasible_starts_converge_to_the_same_weighted_endpoint() {
        let reference = reference_input(2.5);
        let request = QeconWeightedProjectionInput {
            reference: reference.clone(),
            target_q_econ: 1.5,
            bounds: projection_bounds(
                &reference,
                Some(QeconParameter::FusionPowerDensity),
                Some(QeconParameter::NetPriceOfEnergy),
                1.0,
                1.0,
            ),
        };
        let bounds = order_projection_bounds(&request.bounds).unwrap();
        let corner = favourable_corner(&bounds);
        let clamped = clamp_reference(&input_values(&reference), &bounds);
        let near_boundary =
            strict_feasible_start(&reference, &corner, &clamped, request.target_q_econ).unwrap();
        let starts = [
            interpolate(&corner, &near_boundary, 0.2),
            interpolate(&corner, &near_boundary, 0.5),
            interpolate(&corner, &near_boundary, 0.8),
        ];
        let mut objectives = Vec::new();
        for start in starts {
            let (values, _, converged, _) = optimize_fixed_lifetime(
                &reference,
                &bounds,
                &start,
                request.target_q_econ,
            )
            .unwrap();
            assert!(converged);
            assert!(q_at_values(&reference, &values).unwrap() >= request.target_q_econ);
            let input = input_from_values(&reference, &values).unwrap();
            objectives.push(weighted_distance_squared(&input, &reference, &bounds));
        }
        let minimum = objectives.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum = objectives.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((maximum - minimum) / minimum.max(1e-12) < 2e-3);
    }

    #[test]
    fn rejects_infeasible_box_and_invalid_convexity_condition() {
        let reference = reference_input(2.5);
        let mut infeasible = request(reference.clone(), 1e6);
        infeasible.bounds = projection_bounds(
            &reference,
            Some(QeconParameter::FusionPowerDensity),
            None,
            1.0,
            1.0,
        );
        assert!(project_qecon_to_viability(&infeasible).is_err());

        let mut invalid = request(reference.clone(), 1.5);
        invalid.bounds = projection_bounds(
            &reference,
            Some(QeconParameter::FusionPowerDensity),
            None,
            1.0,
            1.0,
        );
        invalid
            .bounds
            .iter_mut()
            .find(|bound| bound.parameter == QeconParameter::FusionPowerDensity)
            .unwrap()
            .minimum = 0.5;
        assert!(project_qecon_to_viability(&invalid).is_err());
    }

    #[test]
    fn projection_is_deterministic_and_keeps_integer_lifetime() {
        let reference = reference_input(2.5);
        let mut request = request(reference, 1.5);
        request.bounds = projection_bounds(
            &request.reference,
            Some(QeconParameter::FusionPowerDensity),
            None,
            1.0,
            1.0,
        );
        let first = project_qecon_to_viability(&request).unwrap();
        let second = project_qecon_to_viability(&request).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.projected_input.design.plant_lifetime_years.fract(),
            0.0
        );
    }
}
