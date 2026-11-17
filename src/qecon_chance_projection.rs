//! Chance-constrained weighted Q_econ projection.
//!
//! This is an approximate sample-average coordinate search over the weighted
//! closest-design objective. It uses common random numbers for all candidate
//! evaluations in the optimization ensemble, and reserves a separate seeded
//! ensemble for validation after the candidate has been selected.
//!
//! The chance constraint is enforced with the Wilson lower 95% bound on the
//! estimated probability P(Q_econ >= target), not with the point estimate.
//! This controls finite Monte Carlo sampling error only; it does not address
//! misspecified distributions, epistemic uncertainty or omitted physics.
//!
//! The algorithm enumerates the integer plant-life interval and repeatedly
//! moves each continuous parameter toward its clamped reference value using
//! bisection on the fixed optimization ensemble. It is deliberately reported
//! as an approximate coordinate search, not a globally optimal optimizer.

use crate::qecon::{
    QECON_MODEL_REVISION, QeconEvidenceClass, QeconInput, calculate_qecon,
};
use crate::qecon_exploration::QeconParameter;
use crate::qecon_projection::{
    QeconProjectionBound, QeconProjectionBoundState, QeconProjectionParameterResult,
    bound_state, clamp_reference, favourable_corner, input_from_values, input_values,
    order_projection_bounds, validate_projection_input, weighted_distance_squared,
};
use crate::qecon_robust::{
    QeconRobustAssessmentScenario, QeconRobustRunEstimate,
    run_robust_ensemble, validate_robust_assessment_scenario,
};
use serde::{Deserialize, Serialize};

pub const QECON_CHANCE_PROJECTION_REVISION: &str = "qecon-chance-projection-v0.1";
pub const MAX_CHANCE_COORDINATE_SWEEPS: u32 = 50;
pub const MAX_CHANCE_CANDIDATE_EVALUATIONS: u32 = 100_000;
const BISECTION_STEPS: usize = 28;
const COORDINATE_RELATIVE_TOLERANCE: f64 = 2e-6;

/// Chance-projection request: robust-viability model plus weighted design box.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconChanceProjectionInput {
    pub robust_assessment: QeconRobustAssessmentScenario,
    pub bounds: Vec<QeconProjectionBound>,
    pub max_coordinate_sweeps: u32,
    pub max_candidate_evaluations: u32,
}

/// Status of the approximate optimization search.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconChanceProjectionSearchStatus {
    CoordinateSearchConverged,
    CoordinateSweepLimit,
    CandidateEvaluationBudgetExhausted,
}

/// Result of a chance-constrained weighted projection with independent holdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconChanceProjectionResult {
    pub model_revision: String,
    pub robust_assessment_revision: String,
    pub chance_projection_revision: String,
    pub scenario_id: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    /// Digest for the baseline scenario only. The projected candidate must be
    /// canonically serialized and hashed by the shared evidence pipeline.
    pub base_input_digest_sha256: String,
    pub uncertainty_input_digest_sha256: String,
    pub nominal_reference_input: QeconInput,
    pub projected_input: QeconInput,
    pub target_q_econ: f64,
    pub required_viability_probability: f64,
    pub projected_nominal_q_econ: f64,
    pub weighted_distance_squared: f64,
    pub optimization_probability_margin: f64,
    pub validation_probability_margin: f64,
    pub optimization: QeconRobustRunEstimate,
    pub validation: QeconRobustRunEstimate,
    pub validation_qualified: bool,
    pub search_status: QeconChanceProjectionSearchStatus,
    pub integer_lifetime_candidates_examined: u32,
    pub integer_lifetimes_with_robust_feasible_corner: u32,
    pub coordinate_sweeps_for_selected_lifetime: u32,
    pub total_coordinate_sweeps: u64,
    pub optimization_candidate_evaluations: u32,
    pub validation_model_evaluations: u64,
    pub parameter_results: Vec<QeconProjectionParameterResult>,
    pub assumptions: Vec<String>,
}

/// Search for a weighted design whose Monte Carlo Wilson lower bound meets
/// the requested Q_econ viability probability, then validate on an independent
/// sample seed.
///
/// **Common random numbers:** every candidate uses the same optimization seed
/// and sample count. Candidate comparisons are therefore paired by draw index.
/// **Independent validation:** the validation seed is only used after selecting
/// the candidate. It is not consulted during coordinate search.
///
/// The result is an approximate coordinate-search candidate. It must not be
/// described as a globally optimal design until independently benchmarked
/// against other optimizers and multiple starts.
pub fn project_qecon_to_chance_constrained_viability(
    request: &QeconChanceProjectionInput,
) -> Result<QeconChanceProjectionResult, String> {
    let robust = &request.robust_assessment;
    validate_robust_assessment_scenario(robust)?;
    let weighted = crate::qecon_projection::QeconWeightedProjectionInput {
        reference: robust.base_scenario.input.clone(),
        target_q_econ: robust.target_q_econ,
        bounds: request.bounds.clone(),
    };
    validate_projection_input(&weighted)?;
    if request.max_coordinate_sweeps == 0
        || request.max_coordinate_sweeps > MAX_CHANCE_COORDINATE_SWEEPS
    {
        return Err(format!(
            "max_coordinate_sweeps must be in 1..={MAX_CHANCE_COORDINATE_SWEEPS}"
        ));
    }
    if request.max_candidate_evaluations == 0
        || request.max_candidate_evaluations > MAX_CHANCE_CANDIDATE_EVALUATIONS
    {
        return Err(format!(
            "max_candidate_evaluations must be in 1..={MAX_CHANCE_CANDIDATE_EVALUATIONS}"
        ));
    }

    let bounds = order_projection_bounds(&request.bounds)?;
    let reference_values = input_values(&weighted.reference);
    let lifetime_bound = &bounds[QeconParameter::PlantLifetime.index()];
    let first_lifetime = lifetime_bound.minimum as u32;
    let last_lifetime = lifetime_bound.maximum as u32;

    let mut candidate_evaluations = 0_u32;
    let mut total_coordinate_sweeps = 0_u64;
    let mut lifetime_candidates_examined = 0_u32;
    let mut robust_feasible_lifetime_count = 0_u32;
    let mut budget_exhausted = false;
    let mut best: Option<ChanceCandidate> = None;

    'lifetimes: for lifetime in first_lifetime..=last_lifetime {
        lifetime_candidates_examined += 1;
        let mut current = favourable_corner(&bounds);
        current[QeconParameter::PlantLifetime.index()] = f64::from(lifetime);

        let Some(mut current_estimate) = evaluate_optimization_candidate(
            &weighted.reference,
            &current,
            robust,
            &mut candidate_evaluations,
            request.max_candidate_evaluations,
        )? else {
            if candidate_evaluations >= request.max_candidate_evaluations {
                budget_exhausted = true;
                break;
            }
            continue;
        };

        if !is_chance_feasible(
            &current_estimate,
            robust.required_viability_probability,
        ) {
            continue;
        }
        robust_feasible_lifetime_count += 1;

        let mut converged = false;
        let mut selected_sweeps = 0_u32;
        for sweep in 0..request.max_coordinate_sweeps {
            selected_sweeps = sweep + 1;
            total_coordinate_sweeps += 1;
            let mut maximum_relative_change: f64 = 0.0;

            for parameter in QeconParameter::ALL {
                if parameter == QeconParameter::PlantLifetime {
                    continue;
                }
                let index = parameter.index();
                let target_value = reference_values[index]
                    .clamp(bounds[index].minimum, bounds[index].maximum);
                let current_value = current[index];
                let relative_gap = (target_value - current_value).abs()
                    / current_value.abs().max(bounds[index].scale * 1e-9).max(1e-12);
                if relative_gap <= COORDINATE_RELATIVE_TOLERANCE {
                    continue;
                }

                let mut target_values = current;
                target_values[index] = target_value;
                let target_estimate = evaluate_optimization_candidate(
                    &weighted.reference,
                    &target_values,
                    robust,
                    &mut candidate_evaluations,
                    request.max_candidate_evaluations,
                )?;

                match target_estimate {
                    Some(estimate)
                        if is_chance_feasible(
                            &estimate,
                            robust.required_viability_probability,
                        ) =>
                    {
                        current = target_values;
                        current_estimate = estimate;
                        maximum_relative_change = maximum_relative_change.max(relative_gap);
                    }
                    _ => {
                        // Current is known chance-feasible; the clamped
                        // reference is not. Repeatedly bisect this segment,
                        // always retaining the feasible endpoint.
                        let mut feasible_value = current_value;
                        let mut infeasible_value = target_value;
                        let mut feasible_estimate = current_estimate.clone();

                        for _ in 0..BISECTION_STEPS {
                            if candidate_evaluations >= request.max_candidate_evaluations {
                                budget_exhausted = true;
                                break;
                            }
                            let midpoint = 0.5 * (feasible_value + infeasible_value);
                            let mut trial_values = current;
                            trial_values[index] = midpoint;
                            let trial_estimate = evaluate_optimization_candidate(
                                &weighted.reference,
                                &trial_values,
                                robust,
                                &mut candidate_evaluations,
                                request.max_candidate_evaluations,
                            )?;
                            if trial_estimate.as_ref().is_some_and(|estimate| {
                                is_chance_feasible(
                                    estimate,
                                    robust.required_viability_probability,
                                )
                            }) {
                                feasible_value = midpoint;
                                feasible_estimate = trial_estimate.expect("checked some");
                            } else {
                                infeasible_value = midpoint;
                            }
                            if (feasible_value - infeasible_value).abs()
                                / feasible_value.abs().max(1e-12)
                                <= COORDINATE_RELATIVE_TOLERANCE
                            {
                                break;
                            }
                        }

                        current[index] = feasible_value;
                        current_estimate = feasible_estimate;
                        maximum_relative_change = maximum_relative_change.max(
                            (current_value - feasible_value).abs()
                                / current_value.abs().max(1e-12),
                        );
                        if budget_exhausted {
                            break;
                        }
                    }
                }

                if candidate_evaluations >= request.max_candidate_evaluations {
                    budget_exhausted = true;
                    break;
                }
            }

            if budget_exhausted {
                break;
            }
            if maximum_relative_change <= COORDINATE_RELATIVE_TOLERANCE {
                converged = true;
                break;
            }
        }

        let candidate_input = input_from_values(&weighted.reference, &current)?;
        let objective = weighted_distance_squared(
            &candidate_input,
            &weighted.reference,
            &bounds,
        );
        let candidate = ChanceCandidate {
            input: candidate_input,
            objective,
            search_converged: converged,
            coordinate_sweeps: selected_sweeps,
        };
        if best.as_ref().is_none_or(|incumbent| candidate.objective < incumbent.objective) {
            best = Some(candidate);
        }
        if budget_exhausted {
            break 'lifetimes;
        }
    }

    let best = best.ok_or_else(|| {
        "no chance-feasible design was found inside the declared parameter box and integer-lifetime range".to_string()
    })?;

    // Re-evaluate the chosen point with the fixed optimization seed so the
    // result's optimization estimate corresponds exactly to the returned
    // nominal input, not the last coordinate's cached sample.
    let mut optimization_evaluations = candidate_evaluations;
    let projected_optimization = run_candidate_ensemble(
        &best.input,
        robust,
        robust.uncertainty_input.optimization_seed,
        robust.uncertainty_input.optimization_sample_count,
    )?;
    optimization_evaluations += 1;

    // The validation seed is used for the first time only after candidate
    // selection. It is never consulted by the coordinate-search routine.
    let projected_validation = run_candidate_ensemble(
        &best.input,
        robust,
        robust.uncertainty_input.validation_seed,
        robust.uncertainty_input.validation_sample_count,
    )?;

    let nominal_scenario = {
        let mut scenario = robust.base_scenario.clone();
        scenario.input = best.input.clone();
        scenario
    };
    let nominal_result = crate::qecon::calculate_qecon_scenario(&nominal_scenario)?;
    let validation_qualified = is_chance_feasible(
        &projected_validation,
        robust.required_viability_probability,
    );
    let parameter_results = build_parameter_results(
        &weighted.reference,
        &best.input,
        &bounds,
    );

    let search_status = if budget_exhausted {
        QeconChanceProjectionSearchStatus::CandidateEvaluationBudgetExhausted
    } else if best.search_converged {
        QeconChanceProjectionSearchStatus::CoordinateSearchConverged
    } else {
        QeconChanceProjectionSearchStatus::CoordinateSweepLimit
    };

    Ok(QeconChanceProjectionResult {
        model_revision: QECON_MODEL_REVISION.to_string(),
        robust_assessment_revision: crate::qecon_robust::QECON_ROBUST_ASSESSMENT_REVISION.to_string(),
        chance_projection_revision: QECON_CHANCE_PROJECTION_REVISION.to_string(),
        scenario_id: robust.base_scenario.scenario_id.clone(),
        evidence_class: robust.base_scenario.evidence_class,
        source_locator: robust.base_scenario.source_locator.clone(),
        base_input_digest_sha256: robust.base_scenario.input_digest_sha256.clone(),
        uncertainty_input_digest_sha256: robust.uncertainty_input_digest_sha256.clone(),
        nominal_reference_input: weighted.reference,
        projected_input: best.input,
        target_q_econ: robust.target_q_econ,
        required_viability_probability: robust.required_viability_probability,
        projected_nominal_q_econ: nominal_result.result.q_econ,
        weighted_distance_squared: best.objective,
        optimization_probability_margin: projected_optimization
            .probability_q_econ_at_least_target
            .wilson_lower_95
            - robust.required_viability_probability,
        validation_probability_margin: projected_validation
            .probability_q_econ_at_least_target
            .wilson_lower_95
            - robust.required_viability_probability,
        optimization: projected_optimization,
        validation: projected_validation,
        validation_qualified,
        search_status,
        integer_lifetime_candidates_examined: lifetime_candidates_examined,
        integer_lifetimes_with_robust_feasible_corner: robust_feasible_lifetime_count,
        coordinate_sweeps_for_selected_lifetime: best.coordinate_sweeps,
        total_coordinate_sweeps,
        optimization_candidate_evaluations: optimization_evaluations,
        validation_model_evaluations: u64::from(
            robust.uncertainty_input.validation_sample_count,
        ),
        parameter_results,
        assumptions: vec![
            "The search uses the same optimization seed for every candidate (common random numbers).".to_string(),
            "The validation seed is not read during candidate selection and is used only for the final selected candidate.".to_string(),
            "The chance constraint is the Wilson 95% lower bound of the Monte Carlo probability P(Q_econ >= target), not the point estimate.".to_string(),
            "This is an approximate coordinate/bisection search over a sampled chance constraint, not a global-optimality proof.".to_string(),
            "Plant lifetime is enumerated over integer years; it is never relaxed to a continuous variable.".to_string(),
            "The objective uses stakeholder scales and difficulty weights in the original design parameters.".to_string(),
            "The result preserves the baseline input digest. A new digest for the projected candidate must be generated by the shared evidence layer.".to_string(),
            "The probability is conditional on caller-supplied marginals, copula, Q_econ equations and omitted-physics assumptions.".to_string(),
            "Wilson intervals quantify finite sampling error only; they do not include uncertainty in the chosen uncertainty model or physical model discrepancy.".to_string(),
            "A validation-qualified probability is not a physical reactor success probability or a heat-exhaust/material/maintenance qualification.".to_string(),
        ],
    })
}

#[derive(Debug, Clone)]
struct ChanceCandidate {
    input: QeconInput,
    objective: f64,
    search_converged: bool,
    coordinate_sweeps: u32,
}

fn evaluate_optimization_candidate(
    reference: &QeconInput,
    values: &[f64; QECON_PARAMETER_COUNT],
    robust: &QeconRobustAssessmentScenario,
    evaluations: &mut u32,
    max_evaluations: u32,
) -> Result<Option<QeconRobustRunEstimate>, String> {
    if *evaluations >= max_evaluations {
        return Ok(None);
    }
    let candidate_input = input_from_values(reference, values)?;
    match run_candidate_ensemble(
        &candidate_input,
        robust,
        robust.uncertainty_input.optimization_seed,
        robust.uncertainty_input.optimization_sample_count,
    ) {
        Ok(result) => {
            *evaluations += 1;
            Ok(Some(result))
        }
        Err(error) if error.contains("invalid input") || error.contains("outside") => Ok(None),
        Err(error) => Err(error),
    }
}

fn run_candidate_ensemble(
    input: &QeconInput,
    robust: &QeconRobustAssessmentScenario,
    seed: u64,
    sample_count: u32,
) -> Result<QeconRobustRunEstimate, String> {
    let mut candidate_scenario = robust.clone();
    candidate_scenario.base_scenario.input = input.clone();
    validate_robust_assessment_scenario(&candidate_scenario)?;
    run_robust_ensemble(
        input,
        robust.target_q_econ,
        &robust.uncertainty_input,
        seed,
        sample_count,
    )
}

fn is_chance_feasible(estimate: &QeconRobustRunEstimate, required_probability: f64) -> bool {
    estimate.probability_q_econ_at_least_target.wilson_lower_95 >= required_probability
}

fn build_parameter_results(
    reference: &QeconInput,
    projected: &QeconInput,
    bounds: &[QeconProjectionBound; QECON_PARAMETER_COUNT],
) -> Vec<QeconProjectionParameterResult> {
    QeconParameter::ALL
        .into_iter()
        .map(|parameter| {
            let index = parameter.index();
            let bound = &bounds[index];
            let reference_value = parameter.value(reference);
            let projected_value = parameter.value(projected);
            QeconProjectionParameterResult {
                parameter,
                unit: parameter.unit().to_string(),
                reference_value,
                projected_value,
                minimum: bound.minimum,
                maximum: bound.maximum,
                scale: bound.scale,
                difficulty_weight: bound.difficulty_weight,
                weighted_distance_contribution: bound.difficulty_weight
                    * ((projected_value - reference_value) / bound.scale).powi(2),
                bound_state: bound_state(projected_value, bound),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qecon::{QECON_MODEL_REVISION, QeconDesignParameters};
    use crate::qecon_robust::{QeconRobustMarginal, QeconRobustTransform, QeconRobustUncertaintyInput};

    fn base_input() -> QeconInput {
        QeconInput {
            fusion_power_density_mw_per_m2: 2.5,
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

    fn fixed_marginals() -> Vec<QeconRobustMarginal> {
        QeconParameter::ALL
            .into_iter()
            .map(|parameter| {
                let transform = if parameter == QeconParameter::TargetCostPerEnergyYield
                    || parameter == QeconParameter::RealInterestRate
                    || parameter == QeconParameter::PlantLifetime
                {
                    QeconRobustTransform::Additive
                } else {
                    QeconRobustTransform::Multiplicative
                };
                let value = if transform == QeconRobustTransform::Multiplicative {
                    1.0
                } else {
                    0.0
                };
                QeconRobustMarginal {
                    parameter,
                    transform,
                    minimum: value,
                    mode: value,
                    maximum: value,
                }
            })
            .collect()
    }

    fn identity() -> [[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT] {
        let mut matrix = [[0.0; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT];
        for (index, row) in matrix.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        matrix
    }

    fn bounds(reference: &QeconInput) -> Vec<QeconProjectionBound> {
        QeconParameter::ALL
            .into_iter()
            .map(|parameter| {
                let value = parameter.value(reference);
                let (minimum, maximum, scale) = if parameter == QeconParameter::FusionPowerDensity {
                    (value * 0.55, 6.0, 3.5)
                } else if value == 0.0 {
                    (1e-9, 1e-9, 1e-9)
                } else {
                    (value, value, value.max(1e-3))
                };
                QeconProjectionBound {
                    parameter,
                    minimum,
                    maximum,
                    scale,
                    difficulty_weight: 1.0,
                }
            })
            .collect()
    }

    fn request() -> QeconChanceProjectionInput {
        let input = base_input();
        QeconChanceProjectionInput {
            robust_assessment: QeconRobustAssessmentScenario {
                base_scenario: crate::qecon::QeconScenario {
                    scenario_id: "chance-projection-fixture".to_string(),
                    model_revision: QECON_MODEL_REVISION.to_string(),
                    evidence_class: QeconEvidenceClass::ScenarioAssumption,
                    source_locator: None,
                    input_digest_sha256: "a".repeat(64),
                    assumptions: vec!["Illustrative base design".to_string()],
                    input: input.clone(),
                },
                target_q_econ: 1.5,
                required_viability_probability: 0.9,
                uncertainty_input_digest_sha256: "b".repeat(64),
                uncertainty_input: QeconRobustUncertaintyInput {
                    optimization_sample_count: 128,
                    validation_sample_count: 256,
                    optimization_seed: 101,
                    validation_seed: 202,
                    marginals: fixed_marginals(),
                    latent_gaussian_correlation: identity(),
                },
                uncertainty_assumptions: vec![
                    "Fixed perturbations exercise the deterministic limit.".to_string(),
                ],
            },
            bounds: bounds(&input),
            max_coordinate_sweeps: 3,
            max_candidate_evaluations: 2_000,
        }
    }

    #[test]
    fn chance_projection_hits_required_probability_and_independently_validates() {
        let result = project_qecon_to_chance_constrained_viability(&request()).unwrap();
        assert!(result.optimization_probability_margin >= 0.0);
        assert!(result.validation_probability_margin >= 0.0);
        assert!(result.validation_qualified);
        assert!(result.projected_nominal_q_econ >= 1.5);
        assert!(result.projected_input.fusion_power_density_mw_per_m2 > 2.5);
        assert_ne!(result.optimization.seed, result.validation.seed);
    }

    #[test]
    fn chance_projection_uses_wilson_lower_bound_not_point_estimate() {
        let mut input = request();
        input.robust_assessment.required_viability_probability = 0.99;
        input.robust_assessment.uncertainty_input.optimization_sample_count = 128;
        input.robust_assessment.uncertainty_input.validation_sample_count = 256;
        let result = project_qecon_to_chance_constrained_viability(&input);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_reused_seeds_and_incomplete_projection_bounds() {
        let mut input = request();
        input.robust_assessment.uncertainty_input.validation_seed = 101;
        assert!(project_qecon_to_chance_constrained_viability(&input).is_err());

        input = request();
        input.bounds.pop();
        assert!(project_qecon_to_chance_constrained_viability(&input).is_err());
    }

    #[test]
    fn candidate_evaluation_budget_is_reported_without_falsely_claiming_convergence() {
        let mut input = request();
        input.max_candidate_evaluations = 1;
        let result = project_qecon_to_chance_constrained_viability(&input).unwrap();
        assert_eq!(
            result.search_status,
            QeconChanceProjectionSearchStatus::CandidateEvaluationBudgetExhausted
        );
    }
}
