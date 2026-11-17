//! Decision exploration for Q_econ: local sensitivity and correlated uncertainty.
//!
//! The deterministic Q_econ kernel remains in `qecon`. This module adds:
//! - local finite-difference sensitivity estimates for all ten control inputs;
//! - seeded Monte Carlo using triangular marginals and a Gaussian copula;
//! - probability and quantile summaries for economic viability;
//! - complete sampled inputs and outputs for audit/replay.
//!
//! The uncertainty layer is a scenario-analysis tool, not a calibrated forecast.
//! Marginal ranges, modes and the latent Gaussian correlation matrix are
//! assumptions supplied by the caller. A reproducible seed makes the draw
//! sequence replayable; it does not make the assumptions empirically valid.
//!
//! The correlation matrix is for latent standard-normal variables in a Gaussian
//! copula, in the fixed parameter order defined by `QeconParameter`. It is not
//! a Pearson correlation matrix for the physical/economic parameters after
//! applying triangular marginal transforms.

use crate::qecon::{
    QECON_MODEL_REVISION, QeconEvidenceClass, QeconInput, QeconResult, QeconScenario,
    QeconScenarioResult, calculate_qecon, calculate_qecon_scenario,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const QECON_EXPLORATION_REVISION: &str = "qecon-exploration-v0.1";
pub const QECON_PARAMETER_COUNT: usize = 10;
pub const MAX_QECON_UNCERTAINTY_SAMPLES: u32 = 100_000;

/// The ten controlling parameters in the order used by the correlation matrix.
///
/// The order is fixed and independent of the order in which marginals are
/// supplied: power density, replacement time, fluence limit, net efficiency,
/// net energy price, target cost per yield, surface replacement cost, plant
/// areal cost, real interest rate, plant lifetime.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconParameter {
    FusionPowerDensity,
    SurfaceReplacementTime,
    SurfaceEnergyFluenceLimit,
    NetEnergyConversionEfficiency,
    NetPriceOfEnergy,
    TargetCostPerEnergyYield,
    SurfaceReplacementCost,
    PlantArealCost,
    RealInterestRate,
    PlantLifetime,
}

impl QeconParameter {
    pub const ALL: [Self; QECON_PARAMETER_COUNT] = [
        Self::FusionPowerDensity,
        Self::SurfaceReplacementTime,
        Self::SurfaceEnergyFluenceLimit,
        Self::NetEnergyConversionEfficiency,
        Self::NetPriceOfEnergy,
        Self::TargetCostPerEnergyYield,
        Self::SurfaceReplacementCost,
        Self::PlantArealCost,
        Self::RealInterestRate,
        Self::PlantLifetime,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::FusionPowerDensity => 0,
            Self::SurfaceReplacementTime => 1,
            Self::SurfaceEnergyFluenceLimit => 2,
            Self::NetEnergyConversionEfficiency => 3,
            Self::NetPriceOfEnergy => 4,
            Self::TargetCostPerEnergyYield => 5,
            Self::SurfaceReplacementCost => 6,
            Self::PlantArealCost => 7,
            Self::RealInterestRate => 8,
            Self::PlantLifetime => 9,
        }
    }

    pub const fn unit(self) -> &'static str {
        match self {
            Self::FusionPowerDensity => "MW/m²",
            Self::SurfaceReplacementTime => "calendar years",
            Self::SurfaceEnergyFluenceLimit => "MW·year/m²",
            Self::NetEnergyConversionEfficiency => "fraction",
            Self::NetPriceOfEnergy => "USD/MWh",
            Self::TargetCostPerEnergyYield => "USD/MJ",
            Self::SurfaceReplacementCost => "MUSD/m²",
            Self::PlantArealCost => "MUSD/m²",
            Self::RealInterestRate => "percentage points",
            Self::PlantLifetime => "years",
        }
    }

    pub fn value(self, input: &QeconInput) -> f64 {
        match self {
            Self::FusionPowerDensity => input.fusion_power_density_mw_per_m2,
            Self::SurfaceReplacementTime => input.design.surface_replacement_time_years,
            Self::SurfaceEnergyFluenceLimit => {
                input.design.surface_energy_fluence_limit_mw_year_per_m2
            }
            Self::NetEnergyConversionEfficiency => input.design.net_energy_conversion_efficiency,
            Self::NetPriceOfEnergy => input.design.net_price_of_energy_usd_per_mwh,
            Self::TargetCostPerEnergyYield => input.design.target_cost_per_energy_yield_usd_per_mj,
            Self::SurfaceReplacementCost => input.design.surface_replacement_cost_musd_per_m2,
            Self::PlantArealCost => input.design.plant_areal_cost_musd_per_m2,
            Self::RealInterestRate => input.design.real_interest_rate_percent,
            Self::PlantLifetime => f64::from(input.design.plant_lifetime_years),
        }
    }

    pub fn with_value(self, input: &QeconInput, value: f64) -> Result<QeconInput, String> {
        if !value.is_finite() {
            return Err(format!("{self:?} must be finite"));
        }
        let mut output = input.clone();
        match self {
            Self::FusionPowerDensity => output.fusion_power_density_mw_per_m2 = value,
            Self::SurfaceReplacementTime => {
                output.design.surface_replacement_time_years = value;
            }
            Self::SurfaceEnergyFluenceLimit => {
                output.design.surface_energy_fluence_limit_mw_year_per_m2 = value;
            }
            Self::NetEnergyConversionEfficiency => {
                output.design.net_energy_conversion_efficiency = value;
            }
            Self::NetPriceOfEnergy => {
                output.design.net_price_of_energy_usd_per_mwh = value;
            }
            Self::TargetCostPerEnergyYield => {
                output.design.target_cost_per_energy_yield_usd_per_mj = value;
            }
            Self::SurfaceReplacementCost => {
                output.design.surface_replacement_cost_musd_per_m2 = value;
            }
            Self::PlantArealCost => output.design.plant_areal_cost_musd_per_m2 = value,
            Self::RealInterestRate => {
                output.design.real_interest_rate_percent = value;
            }
            Self::PlantLifetime => {
                if value < 1.0 || value.fract() != 0.0 || value > f64::from(u32::MAX) {
                    return Err("plant_lifetime_years must be a positive integer".to_string());
                }
                output.design.plant_lifetime_years = value as u32;
            }
        }
        Ok(output)
    }
}

/// How a local sensitivity derivative was evaluated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DifferenceMethod {
    Central,
    Forward,
    Backward,
}

/// Local sensitivity of Q_econ to one control parameter.
///
/// `derivative_per_unit` retains the parameter's native units. The
/// dimensionless elasticity is the more useful cross-parameter comparison
/// when it exists. Elasticity is withheld when either the base parameter or
/// base Q_econ is zero.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconSensitivityEstimate {
    pub parameter: QeconParameter,
    pub base_value: f64,
    pub unit: String,
    pub derivative_per_unit: f64,
    pub relative_elasticity: Option<f64>,
    pub finite_difference_step: f64,
    pub method: DifferenceMethod,
}

/// Estimate and rank local Q_econ sensitivities for all ten parameters.
///
/// Continuous parameters use a relative central difference where the domain
/// permits it, falling back to a one-sided difference at a boundary. Plant
/// lifetime is discrete and is evaluated with a one-year central difference
/// (or one-year forward difference at its lower bound).
///
/// The returned order is descending absolute elasticity. Entries with
/// undefined elasticity (e.g. a zero-valued target-cost assumption) are kept
/// at the end; their native-unit derivative is still reported.
pub fn rank_qecon_sensitivities(
    input: &QeconInput,
) -> Result<Vec<QeconSensitivityEstimate>, String> {
    let base_result = calculate_qecon(input)?;
    let base_q = base_result.q_econ;
    let mut estimates = Vec::with_capacity(QECON_PARAMETER_COUNT);

    for parameter in QeconParameter::ALL {
        let base_value = parameter.value(input);
        let step = match parameter {
            QeconParameter::PlantLifetime => 1.0,
            QeconParameter::RealInterestRate => 1e-4 * base_value.abs().max(1.0),
            _ => 1e-4 * base_value.abs().max(1e-6),
        };

        let plus_input = parameter.with_value(input, base_value + step);
        let minus_input = parameter.with_value(input, base_value - step);

        let plus_q = plus_input
            .as_ref()
            .ok()
            .and_then(|candidate| calculate_qecon(candidate).ok())
            .map(|result| result.q_econ);
        let minus_q = minus_input
            .as_ref()
            .ok()
            .and_then(|candidate| calculate_qecon(candidate).ok())
            .map(|result| result.q_econ);

        let (derivative_per_unit, actual_step, method) = match (plus_q, minus_q) {
            (Some(plus), Some(minus)) => (
                (plus - minus) / (2.0 * step),
                step,
                DifferenceMethod::Central,
            ),
            (Some(plus), None) => ((plus - base_q) / step, step, DifferenceMethod::Forward),
            (None, Some(minus)) => ((base_q - minus) / step, step, DifferenceMethod::Backward),
            (None, None) => {
                return Err(format!(
                    "could not evaluate a valid finite difference for {parameter:?}"
                ));
            }
        };

        if !derivative_per_unit.is_finite() {
            return Err(format!(
                "sensitivity derivative for {parameter:?} is not finite"
            ));
        }
        let relative_elasticity = if base_value != 0.0 && base_q != 0.0 {
            let elasticity = derivative_per_unit * base_value / base_q;
            elasticity.is_finite().then_some(elasticity)
        } else {
            None
        };

        estimates.push(QeconSensitivityEstimate {
            parameter,
            base_value,
            unit: parameter.unit().to_string(),
            derivative_per_unit,
            relative_elasticity,
            finite_difference_step: actual_step,
            method,
        });
    }

    estimates.sort_by(
        |left, right| match (left.relative_elasticity, right.relative_elasticity) {
            (Some(a), Some(b)) => b.abs().total_cmp(&a.abs()),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.parameter.index().cmp(&right.parameter.index()),
        },
    );
    Ok(estimates)
}

/// Independent-input Saltelli/Jansen variance-based sensitivity input.
///
/// The standard first-order/total-order estimators require independent input
/// factors. This API requires an explicit caller declaration and samples each
/// triangular marginal independently; it does not accept the correlated
/// Gaussian-copula matrix used by `QeconUncertaintyInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconSobolInput {
    pub sample_count: u32,
    pub seed: u64,
    pub independence_assumption_confirmed: bool,
    pub marginals: Vec<QeconTriangularMarginal>,
}

/// First-order and total-order Sobol estimates for one independent input.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct QeconSobolIndex {
    pub parameter: QeconParameter,
    pub first_order_index: f64,
    pub total_order_index: f64,
}

/// Reproducible independent-input Saltelli/Jansen sensitivity result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconSobolResult {
    pub model_revision: String,
    pub exploration_revision: String,
    pub seed: u64,
    pub sample_count: u32,
    pub model_evaluation_count: u64,
    pub output_variance: f64,
    pub independence_assumption_confirmed: bool,
    pub marginals: Vec<QeconTriangularMarginal>,
    pub assumptions: Vec<String>,
    pub indices: Vec<QeconSobolIndex>,
}

/// Estimate independent-input first-order and total-order Sobol indices.
///
/// Uses two independent base matrices A/B and one A-with-column-i-from-B
/// hybrid per parameter. The Saltelli first-order covariance estimator and
/// Jansen total-order squared-difference estimator are calculated from the
/// same design. Estimates are not clamped to [0, 1]; finite Monte Carlo noise
/// can produce small excursions and those should remain visible.
pub fn estimate_qecon_sobol_indices(
    base_input: &QeconInput,
    sobol: &QeconSobolInput,
) -> Result<QeconSobolResult, String> {
    if !sobol.independence_assumption_confirmed {
        return Err(
            "standard Sobol estimators require an explicit independent-input declaration"
                .to_string(),
        );
    }
    if sobol.sample_count < 128 || sobol.sample_count > MAX_QECON_UNCERTAINTY_SAMPLES {
        return Err(format!(
            "Sobol sample_count must be in 128..={MAX_QECON_UNCERTAINTY_SAMPLES}"
        ));
    }
    calculate_qecon(base_input)?;
    validate_marginals(&sobol.marginals)?;

    let ordered_marginals = order_marginals(&sobol.marginals)?;
    let mut rng = ChaCha8Rng::seed_from_u64(sobol.seed);
    let mut output_a = Vec::with_capacity(sobol.sample_count as usize);
    let mut output_b = Vec::with_capacity(sobol.sample_count as usize);
    let mut first_order_sums = [0.0; QECON_PARAMETER_COUNT];
    let mut total_order_sums = [0.0; QECON_PARAMETER_COUNT];

    for _ in 0..sobol.sample_count {
        let input_a = sample_independent_qecon_input(base_input, &ordered_marginals, &mut rng)?;
        let input_b = sample_independent_qecon_input(base_input, &ordered_marginals, &mut rng)?;
        let output_a_value = calculate_qecon(&input_a)?.q_econ;
        let output_b_value = calculate_qecon(&input_b)?.q_econ;
        output_a.push(output_a_value);
        output_b.push(output_b_value);

        for parameter in QeconParameter::ALL {
            let value_from_b = parameter.value(&input_b);
            let hybrid = parameter.with_value(&input_a, value_from_b)?;
            let hybrid_output = calculate_qecon(&hybrid)?.q_econ;
            first_order_sums[parameter.index()] +=
                output_b_value * (hybrid_output - output_a_value);
            let difference = output_a_value - hybrid_output;
            total_order_sums[parameter.index()] += difference * difference;
        }
    }

    let sample_count = sobol.sample_count as usize;
    let mut combined_outputs = output_a;
    combined_outputs.extend(output_b);
    let output_mean = combined_outputs.iter().sum::<f64>() / combined_outputs.len() as f64;
    let output_variance = combined_outputs
        .iter()
        .map(|value| {
            let difference = value - output_mean;
            difference * difference
        })
        .sum::<f64>()
        / combined_outputs.len() as f64;
    if !output_variance.is_finite() || output_variance <= 0.0 {
        return Err("Sobol indices require non-zero finite output variance".to_string());
    }

    let indices = QeconParameter::ALL
        .into_iter()
        .map(|parameter| {
            let index = parameter.index();
            QeconSobolIndex {
                parameter,
                first_order_index: first_order_sums[index] / sample_count as f64 / output_variance,
                total_order_index: total_order_sums[index]
                    / (2.0 * sample_count as f64)
                    / output_variance,
            }
        })
        .collect();

    Ok(QeconSobolResult {
        model_revision: QECON_MODEL_REVISION.to_string(),
        exploration_revision: QECON_EXPLORATION_REVISION.to_string(),
        seed: sobol.seed,
        sample_count: sobol.sample_count,
        model_evaluation_count: u64::from(sobol.sample_count)
            * (2 + QECON_PARAMETER_COUNT as u64),
        output_variance,
        independence_assumption_confirmed: true,
        marginals: sobol.marginals.clone(),
        assumptions: vec![
            "All ten input factors are sampled independently from the supplied triangular marginals.".to_string(),
            "Classical first-order and total-order variance indices are only interpreted under this independent-input assumption.".to_string(),
            "Finite Monte Carlo error may produce estimates slightly outside [0, 1]; values are not silently clamped.".to_string(),
            "These indices quantify variance contribution under the declared distributions, not causal effects or intervention costs.".to_string(),
        ],
        indices,
    })
}

fn sample_independent_qecon_input(
    base_input: &QeconInput,
    marginals: &[QeconTriangularMarginal; QECON_PARAMETER_COUNT],
    rng: &mut ChaCha8Rng,
) -> Result<QeconInput, String> {
    let mut sampled = base_input.clone();
    for parameter in QeconParameter::ALL {
        let uniform = rng.gen_range(0.0..1.0);
        let mut value = marginals[parameter.index()].inverse_cdf(uniform);
        if parameter == QeconParameter::PlantLifetime {
            value = value.round();
        }
        sampled = parameter.with_value(&sampled, value)?;
    }
    Ok(sampled)
}

/// Global association between one sampled Q_econ input and the Q_econ output.
///
/// Pearson correlation measures linear association; Spearman correlation
/// measures monotonic rank association. These are descriptive associations
/// over the supplied uncertainty ensemble. With correlated inputs they are
/// not independent-input sensitivity indices, causal effects or Sobol indices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconGlobalAssociation {
    pub parameter: QeconParameter,
    pub unit: String,
    pub pearson_correlation: Option<f64>,
    pub spearman_rank_correlation: Option<f64>,
}

/// Rank all ten parameters by absolute Spearman association with Q_econ.
///
/// This evaluates the actual sampled ensemble, so rankings depend on the
/// caller-supplied marginal distributions, dependence assumptions and seed.
/// A constant parameter or constant output produces an undefined correlation,
/// retained as `None` rather than converted to zero.
pub fn rank_qecon_global_associations(
    uncertainty_result: &QeconUncertaintyResult,
) -> Result<Vec<QeconGlobalAssociation>, String> {
    if uncertainty_result.samples.len() < 2
        || uncertainty_result.samples.len() != uncertainty_result.sample_count as usize
    {
        return Err(
            "global associations require at least two complete samples matching sample_count"
                .to_string(),
        );
    }
    let output_values: Vec<f64> = uncertainty_result
        .samples
        .iter()
        .map(|sample| sample.result.q_econ)
        .collect();
    if output_values.iter().any(|value| !value.is_finite()) {
        return Err("global association output values must be finite".to_string());
    }
    let output_ranks = average_ranks(&output_values);
    let mut associations = Vec::with_capacity(QECON_PARAMETER_COUNT);

    for parameter in QeconParameter::ALL {
        let input_values: Vec<f64> = uncertainty_result
            .samples
            .iter()
            .map(|sample| parameter.value(&sample.input))
            .collect();
        if input_values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "global association values for {parameter:?} must be finite"
            ));
        }
        let input_ranks = average_ranks(&input_values);
        associations.push(QeconGlobalAssociation {
            parameter,
            unit: parameter.unit().to_string(),
            pearson_correlation: pearson_correlation(&input_values, &output_values),
            spearman_rank_correlation: pearson_correlation(&input_ranks, &output_ranks),
        });
    }

    associations.sort_by(|left, right| {
        match (
            left.spearman_rank_correlation,
            right.spearman_rank_correlation,
        ) {
            (Some(left_value), Some(right_value)) => right_value.abs().total_cmp(&left_value.abs()),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.parameter.index().cmp(&right.parameter.index()),
        }
    });
    Ok(associations)
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut ordered_indices: Vec<usize> = (0..values.len()).collect();
    ordered_indices.sort_by(|left, right| values[*left].total_cmp(&values[*right]));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < ordered_indices.len() {
        let first_value = values[ordered_indices[start]];
        let mut end = start + 1;
        while end < ordered_indices.len() && values[ordered_indices[end]] == first_value {
            end += 1;
        }
        // Ranks are 1-based; tied values receive their average rank.
        let average_rank = ((start + 1) as f64 + end as f64) / 2.0;
        for ordered_index in &ordered_indices[start..end] {
            ranks[*ordered_index] = average_rank;
        }
        start = end;
    }
    ranks
}

fn pearson_correlation(left: &[f64], right: &[f64]) -> Option<f64> {
    if left.len() != right.len()
        || left.len() < 2
        || left.iter().all(|value| *value == left[0])
        || right.iter().all(|value| *value == right[0])
    {
        return None;
    }
    let left_mean = left.iter().sum::<f64>() / left.len() as f64;
    let right_mean = right.iter().sum::<f64>() / right.len() as f64;
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;
    for (left_value, right_value) in left.iter().zip(right) {
        let left_delta = left_value - left_mean;
        let right_delta = right_value - right_mean;
        covariance += left_delta * right_delta;
        left_variance += left_delta * left_delta;
        right_variance += right_delta * right_delta;
    }
    let denominator = (left_variance * right_variance).sqrt();
    if !denominator.is_finite() || denominator <= 0.0 {
        return None;
    }
    let correlation = covariance / denominator;
    correlation
        .is_finite()
        .then_some(correlation.clamp(-1.0, 1.0))
}

/// Triangular marginal distribution for one Q_econ parameter.
///
/// The distribution is defined by minimum, mode and maximum. Equal bounds
/// express a fixed parameter. Bounds and the mode must remain inside the
/// parameter's mathematical domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconTriangularMarginal {
    pub parameter: QeconParameter,
    pub minimum: f64,
    pub mode: f64,
    pub maximum: f64,
}

impl QeconTriangularMarginal {
    fn validate(&self) -> Result<(), String> {
        if !self.minimum.is_finite()
            || !self.mode.is_finite()
            || !self.maximum.is_finite()
            || self.minimum > self.mode
            || self.mode > self.maximum
        {
            return Err(format!(
                "invalid triangular marginal for {:?}: require finite minimum <= mode <= maximum",
                self.parameter
            ));
        }

        let valid_domain = match self.parameter {
            QeconParameter::FusionPowerDensity => self.minimum > 0.0,
            QeconParameter::SurfaceReplacementTime => self.minimum >= 0.0,
            QeconParameter::SurfaceEnergyFluenceLimit => self.minimum > 0.0,
            QeconParameter::NetEnergyConversionEfficiency => {
                self.minimum >= 0.0 && self.maximum <= 1.0
            }
            QeconParameter::NetPriceOfEnergy => self.minimum >= 0.0,
            QeconParameter::TargetCostPerEnergyYield => self.minimum >= 0.0,
            QeconParameter::SurfaceReplacementCost => self.minimum >= 0.0,
            QeconParameter::PlantArealCost => self.minimum > 0.0,
            QeconParameter::RealInterestRate => self.minimum > -100.0,
            QeconParameter::PlantLifetime => {
                self.minimum >= 1.0
                    && self.minimum.fract() == 0.0
                    && self.mode.fract() == 0.0
                    && self.maximum.fract() == 0.0
                    && self.maximum <= f64::from(u32::MAX)
            }
        };
        if !valid_domain {
            return Err(format!(
                "triangular marginal for {:?} violates its physical/model domain",
                self.parameter
            ));
        }
        Ok(())
    }

    fn inverse_cdf(&self, uniform: f64) -> f64 {
        if self.minimum == self.maximum {
            return self.minimum;
        }
        let split = (self.mode - self.minimum) / (self.maximum - self.minimum);
        if uniform < split {
            self.minimum
                + ((uniform * (self.maximum - self.minimum) * (self.mode - self.minimum)).max(0.0))
                    .sqrt()
        } else {
            self.maximum
                - (((1.0 - uniform) * (self.maximum - self.minimum) * (self.maximum - self.mode))
                    .max(0.0))
                .sqrt()
        }
    }
}

/// Seeded correlated uncertainty input for all ten Q_econ parameters.
///
/// Marginals must define each parameter exactly once. The correlation matrix
/// is a positive-definite correlation matrix of latent standard-normal
/// variables, in `QeconParameter::ALL` order. The Gaussian copula preserves
/// the triangular marginal shapes while introducing rank dependence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconUncertaintyInput {
    pub sample_count: u32,
    pub seed: u64,
    pub marginals: Vec<QeconTriangularMarginal>,
    pub latent_gaussian_correlation: [[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT],
}

/// One complete sampled input and its deterministic model result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconUncertaintySample {
    pub sample_index: u32,
    pub input: QeconInput,
    pub result: QeconResult,
}

/// Distribution summary for a sampled scalar output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct QeconScalarStatistics {
    pub mean: f64,
    pub sample_standard_deviation: f64,
    pub minimum: f64,
    pub percentile_10: f64,
    pub percentile_50: f64,
    pub percentile_90: f64,
    pub maximum: f64,
}

/// Binomial probability estimate with a Wilson 95% interval.
///
/// The interval measures finite Monte Carlo sampling error only. It does not
/// cover uncertainty in the caller-supplied marginals, copula or Q_econ model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct QeconProbabilityEstimate {
    pub successes: u32,
    pub sample_count: u32,
    pub probability_estimate: f64,
    pub wilson_lower_95: f64,
    pub wilson_upper_95: f64,
}

/// Summary statistics and replayable samples for one seeded uncertainty run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconUncertaintyResult {
    pub model_revision: String,
    pub exploration_revision: String,
    pub seed: u64,
    pub sample_count: u32,
    /// Complete caller-supplied distributions and dependence assumptions.
    pub uncertainty_input: QeconUncertaintyInput,
    /// These are scenario-analysis assumptions, not calibrated probabilities.
    pub assumptions: Vec<String>,
    pub q_econ: QeconScalarStatistics,
    pub utilization_fraction: QeconScalarStatistics,
    pub net_economic_rate_musd_per_m2_year: QeconScalarStatistics,
    pub probability_q_econ_at_least_one: QeconProbabilityEstimate,
    pub probability_q_econ_at_least_one_point_five: QeconProbabilityEstimate,
    pub samples: Vec<QeconUncertaintySample>,
}

/// Evidence-carrying wrapper for a Q_econ uncertainty ensemble.
///
/// The two digests are caller-supplied receipts. This function validates their
/// hexadecimal form and preserves them; it does not perform canonical
/// serialization or cryptographic digest verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconUncertaintyScenario {
    pub base_scenario: QeconScenario,
    pub uncertainty_input_digest_sha256: String,
    pub uncertainty_input: QeconUncertaintyInput,
    pub uncertainty_assumptions: Vec<String>,
}

/// Uncertainty result tied to the original scenario's identity and evidence class.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconUncertaintyScenarioResult {
    pub scenario_id: String,
    pub model_revision: String,
    pub exploration_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub base_input_digest_sha256: String,
    pub uncertainty_input_digest_sha256: String,
    pub base_assumptions: Vec<String>,
    pub uncertainty_assumptions: Vec<String>,
    pub result: QeconUncertaintyResult,
}

/// Run an uncertainty ensemble without losing the base scenario's evidence identity.
///
/// The base Q_econ scenario is validated by the canonical scenario evaluator.
/// The uncertainty-input digest is syntax-checked and carried through; digest
/// construction and verification remain the responsibility of the shared
/// evidence pipeline.
pub fn propagate_qecon_scenario_uncertainty(
    scenario: &QeconUncertaintyScenario,
) -> Result<QeconUncertaintyScenarioResult, String> {
    let base_result: QeconScenarioResult = calculate_qecon_scenario(&scenario.base_scenario)?;
    if scenario.uncertainty_input_digest_sha256.len() != 64
        || !scenario
            .uncertainty_input_digest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(
            "uncertainty_input_digest_sha256 must contain exactly 64 hexadecimal characters"
                .to_string(),
        );
    }
    if scenario.uncertainty_assumptions.is_empty()
        || scenario
            .uncertainty_assumptions
            .iter()
            .any(|assumption| assumption.trim().is_empty())
    {
        return Err(
            "uncertainty scenario must preserve at least one non-empty assumption".to_string(),
        );
    }

    let result =
        propagate_qecon_uncertainty(&scenario.base_scenario.input, &scenario.uncertainty_input)?;
    Ok(QeconUncertaintyScenarioResult {
        scenario_id: base_result.scenario_id,
        model_revision: base_result.model_revision,
        exploration_revision: result.exploration_revision.clone(),
        evidence_class: base_result.evidence_class,
        source_locator: base_result.source_locator,
        base_input_digest_sha256: base_result.input_digest_sha256,
        uncertainty_input_digest_sha256: scenario.uncertainty_input_digest_sha256.clone(),
        base_assumptions: base_result.assumptions,
        uncertainty_assumptions: scenario.uncertainty_assumptions.clone(),
        result,
    })
}

/// Run seeded correlated uncertainty propagation for the Q_econ model.
///
/// A triangular distribution is used for each marginal. Correlations are
/// introduced with a Gaussian copula and Cholesky factorization. All ten input
/// values are retained with every result so the sampled ensemble can be
/// inspected, replayed and exported rather than relying on summary statistics
/// alone.
///
/// This function does not infer distributions from data. The caller must
/// justify each range, mode and dependence assumption and label the resulting
/// ensemble as a scenario/model output.
pub fn propagate_qecon_uncertainty(
    base_input: &QeconInput,
    uncertainty: &QeconUncertaintyInput,
) -> Result<QeconUncertaintyResult, String> {
    if uncertainty.sample_count < 2 || uncertainty.sample_count > MAX_QECON_UNCERTAINTY_SAMPLES {
        return Err(format!(
            "sample_count must be in 2..={MAX_QECON_UNCERTAINTY_SAMPLES}"
        ));
    }

    calculate_qecon(base_input)?;
    validate_marginals(&uncertainty.marginals)?;
    let cholesky = cholesky_correlation(&uncertainty.latent_gaussian_correlation)?;
    let ordered_marginals = order_marginals(&uncertainty.marginals)?;

    let mut rng = ChaCha8Rng::seed_from_u64(uncertainty.seed);
    let mut samples = Vec::with_capacity(uncertainty.sample_count as usize);
    let mut q_values = Vec::with_capacity(uncertainty.sample_count as usize);
    let mut utilization_values = Vec::with_capacity(uncertainty.sample_count as usize);
    let mut net_rate_values = Vec::with_capacity(uncertainty.sample_count as usize);
    let mut viable_count = 0_u32;
    let mut q15_count = 0_u32;

    for sample_index in 0..uncertainty.sample_count {
        let independent_normals = standard_normal_vector(&mut rng);
        let correlated_normals = multiply_lower_triangular(&cholesky, &independent_normals);
        let mut sampled_input = base_input.clone();

        for parameter in QeconParameter::ALL {
            let latent = correlated_normals[parameter.index()];
            let uniform = standard_normal_cdf(latent).clamp(1e-12, 1.0 - 1e-12);
            let mut sampled_value = ordered_marginals[parameter.index()].inverse_cdf(uniform);
            if parameter == QeconParameter::PlantLifetime {
                sampled_value = sampled_value.round();
            }
            sampled_input = parameter.with_value(&sampled_input, sampled_value)?;
        }

        let result = calculate_qecon(&sampled_input).map_err(|error| {
            format!("uncertainty sample {sample_index} produced an invalid input: {error}")
        })?;
        if result.q_econ >= 1.0 {
            viable_count += 1;
        }
        if result.q_econ >= 1.5 {
            q15_count += 1;
        }
        q_values.push(result.q_econ);
        utilization_values.push(result.utilization_fraction);
        net_rate_values.push(result.net_economic_rate_musd_per_m2_year);
        samples.push(QeconUncertaintySample {
            sample_index,
            input: sampled_input,
            result,
        });
    }

    Ok(QeconUncertaintyResult {
        model_revision: QECON_MODEL_REVISION.to_string(),
        exploration_revision: QECON_EXPLORATION_REVISION.to_string(),
        seed: uncertainty.seed,
        sample_count: uncertainty.sample_count,
        uncertainty_input: uncertainty.clone(),
        assumptions: vec![
            "Marginal parameter distributions are caller-supplied triangular assumptions.".to_string(),
            "Dependence is imposed through a Gaussian copula using the supplied latent correlation matrix.".to_string(),
            "The latent correlation matrix is not a Pearson correlation matrix of transformed physical parameters.".to_string(),
            "This seeded ensemble is conditional scenario analysis, not a calibrated forecast or empirical confidence interval.".to_string(),
        ],
        q_econ: scalar_statistics(&q_values)?,
        utilization_fraction: scalar_statistics(&utilization_values)?,
        net_economic_rate_musd_per_m2_year: scalar_statistics(&net_rate_values)?,
        probability_q_econ_at_least_one: wilson_probability_estimate(
            viable_count,
            uncertainty.sample_count,
        ),
        probability_q_econ_at_least_one_point_five: wilson_probability_estimate(
            q15_count,
            uncertainty.sample_count,
        ),
        samples,
    })
}

fn validate_marginals(marginals: &[QeconTriangularMarginal]) -> Result<(), String> {
    if marginals.len() != QECON_PARAMETER_COUNT {
        return Err(format!(
            "exactly {QECON_PARAMETER_COUNT} marginals are required"
        ));
    }
    let mut seen = [false; QECON_PARAMETER_COUNT];
    for marginal in marginals {
        marginal.validate()?;
        let index = marginal.parameter.index();
        if seen[index] {
            return Err(format!("duplicate marginal for {:?}", marginal.parameter));
        }
        seen[index] = true;
    }
    if seen.iter().any(|present| !present) {
        return Err("all ten Q_econ parameters must have one marginal".to_string());
    }
    Ok(())
}

fn order_marginals(
    marginals: &[QeconTriangularMarginal],
) -> Result<[QeconTriangularMarginal; QECON_PARAMETER_COUNT], String> {
    let mut ordered: [Option<QeconTriangularMarginal>; QECON_PARAMETER_COUNT] =
        std::array::from_fn(|_| None);
    for marginal in marginals {
        let index = marginal.parameter.index();
        ordered[index] = Some(marginal.clone());
    }
    let mut values = Vec::with_capacity(QECON_PARAMETER_COUNT);
    for marginal in ordered {
        values.push(marginal.ok_or_else(|| "missing Q_econ marginal".to_string())?);
    }
    values
        .try_into()
        .map_err(|_| "expected ten Q_econ marginals".to_string())
}

fn cholesky_correlation(
    matrix: &[[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT],
) -> Result<[[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT], String> {
    const N: usize = QECON_PARAMETER_COUNT;
    let mut lower = [[0.0; N]; N];
    for row in 0..N {
        for column in 0..N {
            let value = matrix[row][column];
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return Err("correlation entries must be finite and in [-1, 1]".to_string());
            }
            if (matrix[row][column] - matrix[column][row]).abs() > 1e-10 {
                return Err("latent Gaussian correlation matrix must be symmetric".to_string());
            }
        }
        if (matrix[row][row] - 1.0).abs() > 1e-10 {
            return Err("latent Gaussian correlation matrix diagonal must equal one".to_string());
        }
    }

    for row in 0..N {
        for column in 0..=row {
            let mut residual = matrix[row][column];
            for previous in 0..column {
                residual -= lower[row][previous] * lower[column][previous];
            }
            if row == column {
                if !residual.is_finite() || residual <= 1e-12 {
                    return Err(
                        "latent Gaussian correlation matrix must be positive definite".to_string(),
                    );
                }
                lower[row][column] = residual.sqrt();
            } else {
                lower[row][column] = residual / lower[column][column];
            }
        }
    }
    Ok(lower)
}

fn standard_normal_vector(rng: &mut ChaCha8Rng) -> [f64; QECON_PARAMETER_COUNT] {
    let mut values = [0.0; QECON_PARAMETER_COUNT];
    let mut index = 0;
    while index < QECON_PARAMETER_COUNT {
        let first = rng.gen_range(f64::MIN_POSITIVE..1.0);
        let second = rng.gen_range(0.0..1.0);
        let radius = (-2.0 * first.ln()).sqrt();
        let angle = std::f64::consts::TAU * second;
        values[index] = radius * angle.cos();
        if index + 1 < QECON_PARAMETER_COUNT {
            values[index + 1] = radius * angle.sin();
        }
        index += 2;
    }
    values
}

fn multiply_lower_triangular(
    lower: &[[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT],
    values: &[f64; QECON_PARAMETER_COUNT],
) -> [f64; QECON_PARAMETER_COUNT] {
    let mut output = [0.0; QECON_PARAMETER_COUNT];
    for row in 0..QECON_PARAMETER_COUNT {
        for column in 0..=row {
            output[row] += lower[row][column] * values[column];
        }
    }
    output
}

/// Normal CDF approximation (maximum absolute error approximately 1.5e-7).
fn standard_normal_cdf(value: f64) -> f64 {
    let absolute = value.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * absolute);
    let polynomial = t
        * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937 + t * (-1.821_255_978 + t * 1.330_274_429))));
    let density = (-0.5 * absolute * absolute).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let upper_tail = density * polynomial;
    if value >= 0.0 {
        1.0 - upper_tail
    } else {
        upper_tail
    }
}

fn wilson_probability_estimate(successes: u32, sample_count: u32) -> QeconProbabilityEstimate {
    const Z_95: f64 = 1.959_963_984_540_054;
    let n = f64::from(sample_count);
    let observed = f64::from(successes) / n;
    let z_squared = Z_95 * Z_95;
    let denominator = 1.0 + z_squared / n;
    let center = (observed + z_squared / (2.0 * n)) / denominator;
    let half_width =
        Z_95 / denominator * (observed * (1.0 - observed) / n + z_squared / (4.0 * n * n)).sqrt();
    let lower = if successes == 0 {
        0.0
    } else {
        (center - half_width).clamp(0.0, 1.0)
    };
    let upper = if successes == sample_count {
        1.0
    } else {
        (center + half_width).clamp(0.0, 1.0)
    };
    QeconProbabilityEstimate {
        successes,
        sample_count,
        probability_estimate: observed,
        wilson_lower_95: lower,
        wilson_upper_95: upper,
    }
}

fn scalar_statistics(values: &[f64]) -> Result<QeconScalarStatistics, String> {
    if values.len() < 2 || values.iter().any(|value| !value.is_finite()) {
        return Err("statistics require at least two finite values".to_string());
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let difference = value - mean;
            difference * difference
        })
        .sum::<f64>()
        / (values.len() - 1) as f64;
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let statistics = QeconScalarStatistics {
        mean,
        sample_standard_deviation: variance.sqrt(),
        minimum: sorted[0],
        percentile_10: interpolated_quantile(&sorted, 0.10),
        percentile_50: interpolated_quantile(&sorted, 0.50),
        percentile_90: interpolated_quantile(&sorted, 0.90),
        maximum: sorted[sorted.len() - 1],
    };
    let outputs = [
        statistics.mean,
        statistics.sample_standard_deviation,
        statistics.minimum,
        statistics.percentile_10,
        statistics.percentile_50,
        statistics.percentile_90,
        statistics.maximum,
    ];
    if outputs.iter().any(|value| !value.is_finite()) {
        return Err("calculated statistics are not finite".to_string());
    }
    Ok(statistics)
}

fn interpolated_quantile(sorted: &[f64], probability: f64) -> f64 {
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f64;
    sorted[lower] * (1.0 - fraction) + sorted[upper] * fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qecon::QeconDesignParameters;

    fn input() -> QeconInput {
        QeconInput {
            fusion_power_density_mw_per_m2: 3.7,
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

    fn marginals() -> Vec<QeconTriangularMarginal> {
        vec![
            QeconTriangularMarginal {
                parameter: QeconParameter::FusionPowerDensity,
                minimum: 2.5,
                mode: 3.7,
                maximum: 5.0,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::SurfaceReplacementTime,
                minimum: 0.05,
                mode: 0.1,
                maximum: 0.2,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::SurfaceEnergyFluenceLimit,
                minimum: 2.0,
                mode: 3.125,
                maximum: 5.0,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::NetEnergyConversionEfficiency,
                minimum: 0.3,
                mode: 0.4,
                maximum: 0.5,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::NetPriceOfEnergy,
                minimum: 80.0,
                mode: 100.0,
                maximum: 130.0,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::TargetCostPerEnergyYield,
                minimum: 0.0,
                mode: 0.0,
                maximum: 0.001,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::SurfaceReplacementCost,
                minimum: 0.2,
                mode: 0.3,
                maximum: 0.5,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::PlantArealCost,
                minimum: 7.0,
                mode: 10.0,
                maximum: 14.0,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::RealInterestRate,
                minimum: 1.0,
                mode: 2.0,
                maximum: 4.0,
            },
            QeconTriangularMarginal {
                parameter: QeconParameter::PlantLifetime,
                minimum: 20.0,
                mode: 30.0,
                maximum: 40.0,
            },
        ]
    }

    fn identity_correlation() -> [[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT] {
        let mut matrix = [[0.0; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT];
        for (index, row) in matrix.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        matrix
    }

    fn uncertainty(seed: u64) -> QeconUncertaintyInput {
        QeconUncertaintyInput {
            sample_count: 128,
            seed,
            marginals: marginals(),
            latent_gaussian_correlation: identity_correlation(),
        }
    }

    #[test]
    fn sensitivity_estimates_cover_all_ten_parameters_and_rank_elasticities() {
        let estimates = rank_qecon_sensitivities(&input()).unwrap();
        assert_eq!(estimates.len(), QECON_PARAMETER_COUNT);
        assert!(
            estimates
                .windows(2)
                .filter_map(|pair| pair[0].relative_elasticity.zip(pair[1].relative_elasticity))
                .all(|(left, right)| left.abs() >= right.abs())
        );
        assert!(estimates.iter().any(|entry| {
            entry.parameter == QeconParameter::FusionPowerDensity
                && entry.relative_elasticity.is_some()
        }));
        assert!(estimates.iter().any(|entry| {
            entry.parameter == QeconParameter::TargetCostPerEnergyYield
                && entry.relative_elasticity.is_none()
        }));
    }

    #[test]
    fn sensitivity_signs_match_economic_direction() {
        let estimates = rank_qecon_sensitivities(&input()).unwrap();
        let find = |parameter| {
            estimates
                .iter()
                .find(|entry| entry.parameter == parameter)
                .unwrap()
                .derivative_per_unit
        };
        assert!(find(QeconParameter::FusionPowerDensity) > 0.0);
        assert!(find(QeconParameter::SurfaceEnergyFluenceLimit) > 0.0);
        assert!(find(QeconParameter::NetEnergyConversionEfficiency) > 0.0);
        assert!(find(QeconParameter::NetPriceOfEnergy) > 0.0);
        assert!(find(QeconParameter::SurfaceReplacementTime) < 0.0);
        assert!(find(QeconParameter::TargetCostPerEnergyYield) < 0.0);
        assert!(find(QeconParameter::SurfaceReplacementCost) < 0.0);
        assert!(find(QeconParameter::PlantArealCost) < 0.0);
        assert!(find(QeconParameter::RealInterestRate) < 0.0);
        assert!(find(QeconParameter::PlantLifetime) > 0.0);
    }

    fn sobol_input(seed: u64, sample_count: u32) -> QeconSobolInput {
        QeconSobolInput {
            sample_count,
            seed,
            independence_assumption_confirmed: true,
            marginals: marginals(),
        }
    }

    #[test]
    fn independent_sobol_rejects_missing_independence_declaration_and_small_samples() {
        let mut design = sobol_input(12, 128);
        design.independence_assumption_confirmed = false;
        assert!(estimate_qecon_sobol_indices(&input(), &design).is_err());
        design.independence_assumption_confirmed = true;
        design.sample_count = 127;
        assert!(estimate_qecon_sobol_indices(&input(), &design).is_err());
    }

    #[test]
    fn independent_sobol_is_reproducible_and_reports_model_evaluations() {
        let design = sobol_input(0x5eed, 512);
        let first = estimate_qecon_sobol_indices(&input(), &design).unwrap();
        let second = estimate_qecon_sobol_indices(&input(), &design).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.indices.len(), QECON_PARAMETER_COUNT);
        assert_eq!(
            first.model_evaluation_count,
            512 * (2 + QECON_PARAMETER_COUNT as u64)
        );
        assert!(first.output_variance > 0.0);
    }

    #[test]
    fn independent_sobol_recovers_single_active_input() {
        let base = input();
        let mut design = sobol_input(0x1234, 4096);
        for marginal in &mut design.marginals {
            let value = marginal.parameter.value(&base);
            marginal.minimum = value;
            marginal.mode = value;
            marginal.maximum = value;
        }
        let efficiency = design
            .marginals
            .iter_mut()
            .find(|marginal| marginal.parameter == QeconParameter::NetEnergyConversionEfficiency)
            .unwrap();
        efficiency.minimum = 0.3;
        efficiency.mode = 0.4;
        efficiency.maximum = 0.5;

        let result = estimate_qecon_sobol_indices(&base, &design).unwrap();
        for index in &result.indices {
            if index.parameter == QeconParameter::NetEnergyConversionEfficiency {
                assert!(
                    (index.first_order_index - 1.0).abs() < 0.08,
                    "single active factor first-order index was {}",
                    index.first_order_index
                );
                assert!((index.total_order_index - 1.0).abs() < 0.08);
            } else {
                assert!(index.first_order_index.abs() < 1e-10);
                assert!(index.total_order_index.abs() < 1e-10);
            }
        }
    }

    #[test]
    fn global_associations_cover_all_parameters_and_rank_the_sampled_ensemble() {
        let result = propagate_qecon_uncertainty(&input(), &uncertainty(0x5eed)).unwrap();
        let associations = rank_qecon_global_associations(&result).unwrap();
        assert_eq!(associations.len(), QECON_PARAMETER_COUNT);
        assert!(associations.windows(2).all(|pair| {
            match (
                pair[0].spearman_rank_correlation,
                pair[1].spearman_rank_correlation,
            ) {
                (Some(left), Some(right)) => left.abs() >= right.abs(),
                (Some(_), None) => true,
                (None, _) => true,
            }
        }));
        assert!(associations.iter().all(|entry| {
            entry
                .pearson_correlation
                .is_none_or(|value| (-1.0..=1.0).contains(&value))
                && entry
                    .spearman_rank_correlation
                    .is_none_or(|value| (-1.0..=1.0).contains(&value))
        }));
    }

    #[test]
    fn global_associations_recover_monotonic_relationship_when_other_inputs_are_fixed() {
        let mut scenario = uncertainty(19);
        let base = input();
        for marginal in &mut scenario.marginals {
            let value = marginal.parameter.value(&base);
            marginal.minimum = value;
            marginal.mode = value;
            marginal.maximum = value;
        }
        let replacement_time = scenario
            .marginals
            .iter_mut()
            .find(|marginal| marginal.parameter == QeconParameter::SurfaceReplacementTime)
            .unwrap();
        replacement_time.minimum = 0.05;
        replacement_time.mode = 0.1;
        replacement_time.maximum = 0.2;

        let result = propagate_qecon_uncertainty(&base, &scenario).unwrap();
        let associations = rank_qecon_global_associations(&result).unwrap();
        let replacement_time_association = associations
            .iter()
            .find(|entry| entry.parameter == QeconParameter::SurfaceReplacementTime)
            .unwrap();
        assert_eq!(
            replacement_time_association.spearman_rank_correlation,
            Some(-1.0)
        );
        assert!(
            associations
                .iter()
                .filter(|entry| { entry.parameter != QeconParameter::SurfaceReplacementTime })
                .all(|entry| {
                    entry.pearson_correlation.is_none() && entry.spearman_rank_correlation.is_none()
                })
        );
    }

    #[test]
    fn global_associations_mark_constant_inputs_and_output_as_undefined() {
        let mut scenario = uncertainty(19);
        let base = input();
        for marginal in &mut scenario.marginals {
            let value = marginal.parameter.value(&base);
            marginal.minimum = value;
            marginal.mode = value;
            marginal.maximum = value;
        }
        let result = propagate_qecon_uncertainty(&base, &scenario).unwrap();
        let associations = rank_qecon_global_associations(&result).unwrap();
        assert!(associations.iter().all(|entry| {
            entry.pearson_correlation.is_none() && entry.spearman_rank_correlation.is_none()
        }));
    }

    #[test]
    fn uncertainty_run_is_deterministic_for_fixed_seed_and_inputs() {
        let scenario = uncertainty(0x5eed);
        let first = propagate_qecon_uncertainty(&input(), &scenario).unwrap();
        let second = propagate_qecon_uncertainty(&input(), &scenario).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.samples.len(), 128);
        assert_eq!(first.uncertainty_input, scenario);
        assert!(first.assumptions.len() >= 3);
        assert!((0.0..=1.0).contains(&first.probability_q_econ_at_least_one.probability_estimate));
        assert!(
            first
                .probability_q_econ_at_least_one_point_five
                .probability_estimate
                <= first.probability_q_econ_at_least_one.probability_estimate
        );
        for estimate in [
            first.probability_q_econ_at_least_one,
            first.probability_q_econ_at_least_one_point_five,
        ] {
            assert!(estimate.wilson_lower_95 <= estimate.probability_estimate);
            assert!(estimate.probability_estimate <= estimate.wilson_upper_95);
            assert!(estimate.wilson_upper_95 - estimate.wilson_lower_95 > 0.0);
        }
    }

    #[test]
    fn uncertainty_samples_respect_triangular_marginal_bounds() {
        let result = propagate_qecon_uncertainty(&input(), &uncertainty(7)).unwrap();
        for sample in result.samples {
            for marginal in marginals() {
                let value = marginal.parameter.value(&sample.input);
                assert!(value >= marginal.minimum && value <= marginal.maximum);
            }
            assert!(
                sample.input.design.plant_lifetime_years >= 20
                    && sample.input.design.plant_lifetime_years <= 40
            );
        }
    }

    #[test]
    fn correlated_latent_inputs_preserve_requested_dependence_direction() {
        let mut scenario = uncertainty(99);
        scenario.latent_gaussian_correlation[0][3] = 0.8;
        scenario.latent_gaussian_correlation[3][0] = 0.8;
        let result = propagate_qecon_uncertainty(&input(), &scenario).unwrap();
        let pairs: Vec<(f64, f64)> = result
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.input.fusion_power_density_mw_per_m2,
                    sample.input.design.net_energy_conversion_efficiency,
                )
            })
            .collect();
        let mean_x = pairs.iter().map(|pair| pair.0).sum::<f64>() / pairs.len() as f64;
        let mean_y = pairs.iter().map(|pair| pair.1).sum::<f64>() / pairs.len() as f64;
        let covariance = pairs
            .iter()
            .map(|pair| (pair.0 - mean_x) * (pair.1 - mean_y))
            .sum::<f64>();
        assert!(covariance > 0.0);
    }

    #[test]
    fn rejects_missing_duplicate_invalid_and_non_positive_definite_inputs() {
        let mut scenario = uncertainty(5);
        scenario.marginals.pop();
        assert!(propagate_qecon_uncertainty(&input(), &scenario).is_err());

        let mut scenario = uncertainty(5);
        scenario.marginals[1].parameter = scenario.marginals[0].parameter;
        assert!(propagate_qecon_uncertainty(&input(), &scenario).is_err());

        let mut scenario = uncertainty(5);
        scenario.latent_gaussian_correlation[0][1] = 0.7;
        assert!(propagate_qecon_uncertainty(&input(), &scenario).is_err());

        let mut scenario = uncertainty(5);
        scenario.latent_gaussian_correlation[0][1] = 1.0;
        scenario.latent_gaussian_correlation[1][0] = 1.0;
        assert!(propagate_qecon_uncertainty(&input(), &scenario).is_err());
    }

    #[test]
    fn uncertainty_scenario_preserves_base_evidence_identity() {
        let scenario = QeconUncertaintyScenario {
            base_scenario: QeconScenario {
                scenario_id: "scenario-2026-a".to_string(),
                model_revision: QECON_MODEL_REVISION.to_string(),
                evidence_class: QeconEvidenceClass::ScenarioAssumption,
                source_locator: None,
                input_digest_sha256: "a".repeat(64),
                assumptions: vec!["Illustrative Q_econ base point".to_string()],
                input: input(),
            },
            uncertainty_input_digest_sha256: "b".repeat(64),
            uncertainty_input: uncertainty(0x5eed),
            uncertainty_assumptions: vec![
                "Triangular input ranges are scenario assumptions.".to_string(),
                "Gaussian copula dependence is an illustrative assumption.".to_string(),
            ],
        };
        let result = propagate_qecon_scenario_uncertainty(&scenario).unwrap();
        assert_eq!(result.scenario_id, "scenario-2026-a");
        assert_eq!(
            result.evidence_class,
            QeconEvidenceClass::ScenarioAssumption
        );
        assert_eq!(result.base_input_digest_sha256, "a".repeat(64));
        assert_eq!(result.uncertainty_input_digest_sha256, "b".repeat(64));
        assert_eq!(result.result.seed, 0x5eed);
        assert_eq!(result.base_assumptions, scenario.base_scenario.assumptions);
        assert_eq!(
            result.uncertainty_assumptions,
            scenario.uncertainty_assumptions
        );
    }

    #[test]
    fn uncertainty_scenario_rejects_invalid_digest_and_missing_assumptions() {
        let mut scenario = QeconUncertaintyScenario {
            base_scenario: QeconScenario {
                scenario_id: "scenario-2026-a".to_string(),
                model_revision: QECON_MODEL_REVISION.to_string(),
                evidence_class: QeconEvidenceClass::ScenarioAssumption,
                source_locator: None,
                input_digest_sha256: "a".repeat(64),
                assumptions: vec!["Illustrative base point".to_string()],
                input: input(),
            },
            uncertainty_input_digest_sha256: "b".repeat(64),
            uncertainty_input: uncertainty(123),
            uncertainty_assumptions: vec!["Assumed triangular marginals".to_string()],
        };
        scenario.uncertainty_input_digest_sha256 = "bad".to_string();
        assert!(propagate_qecon_scenario_uncertainty(&scenario).is_err());

        scenario.uncertainty_input_digest_sha256 = "b".repeat(64);
        scenario.uncertainty_assumptions.clear();
        assert!(propagate_qecon_scenario_uncertainty(&scenario).is_err());
    }

    #[test]
    fn wilson_interval_behaves_at_probability_boundaries() {
        let all_success = wilson_probability_estimate(100, 100);
        assert_eq!(all_success.probability_estimate, 1.0);
        assert!(all_success.wilson_lower_95 < 1.0);
        assert_eq!(all_success.wilson_upper_95, 1.0);

        let no_success = wilson_probability_estimate(0, 100);
        assert_eq!(no_success.probability_estimate, 0.0);
        assert_eq!(no_success.wilson_lower_95, 0.0);
        assert!(no_success.wilson_upper_95 > 0.0);
    }

    #[test]
    fn rejects_unreasonably_large_sample_count() {
        let mut scenario = uncertainty(5);
        scenario.sample_count = MAX_QECON_UNCERTAINTY_SAMPLES + 1;
        assert!(propagate_qecon_uncertainty(&input(), &scenario).is_err());
    }
}
