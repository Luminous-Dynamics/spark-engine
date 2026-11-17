//! Robust Q_econ assessment using separate optimization and validation ensembles.
//!
//! This is a chance-constraint assessment primitive, not yet a design
//! optimizer. It evaluates one nominal design under a declared joint
//! distribution, then re-evaluates it with an independent seed.
//!
//! Each uncertain parameter has either a multiplicative triangular factor or
//! an additive triangular offset in native units. For a zero nominal value,
//! multiplicative uncertainty remains zero; use an additive transform when a
//! zero baseline has meaningful uncertainty.
//!
//! The Gaussian-copula matrix is for latent standard-normal variables in the
//! fixed QeconParameter order, not a Pearson correlation matrix of the
//! transformed physical parameters.

use crate::qecon::{
    QECON_MODEL_REVISION, QeconEvidenceClass, QeconInput, QeconScenario,
    QeconScenarioResult, calculate_qecon, calculate_qecon_scenario,
};
use crate::qecon_exploration::{
    MAX_QECON_UNCERTAINTY_SAMPLES, QECON_PARAMETER_COUNT, QeconParameter,
    QeconProbabilityEstimate, QeconScalarStatistics, cholesky_correlation,
    multiply_lower_triangular, scalar_statistics, standard_normal_cdf,
    standard_normal_vector, wilson_probability_estimate,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const QECON_ROBUST_ASSESSMENT_REVISION: &str = "qecon-robust-assessment-v0.1";

/// How a sampled uncertainty value perturbs one nominal design parameter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconRobustTransform {
    /// Sampled triangular value is a positive dimensionless multiplier.
    Multiplicative,
    /// Sampled triangular value is an additive offset in the parameter's units.
    Additive,
}

/// Triangular uncertainty marginal for one design parameter.
///
/// For Multiplicative, min/mode/max are positive factors.
/// For Additive, min/mode/max are signed offsets in native units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconRobustMarginal {
    pub parameter: QeconParameter,
    pub transform: QeconRobustTransform,
    pub minimum: f64,
    pub mode: f64,
    pub maximum: f64,
}

impl QeconRobustMarginal {
    fn validate(&self) -> Result<(), String> {
        if !self.minimum.is_finite()
            || !self.mode.is_finite()
            || !self.maximum.is_finite()
            || self.minimum > self.mode
            || self.mode > self.maximum
        {
            return Err(format!(
                "invalid robust uncertainty marginal for {:?}: require finite minimum <= mode <= maximum",
                self.parameter
            ));
        }
        if self.transform == QeconRobustTransform::Multiplicative && self.minimum <= 0.0 {
            return Err(format!(
                "multiplicative uncertainty factors for {:?} must be positive",
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

/// Separate Monte Carlo budgets and seeds for optimization and validation.
///
/// Candidate comparisons should reuse the same optimization seed (common
/// random numbers). Validation must use a different seed and remain untouched
/// while choosing the candidate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconRobustUncertaintyInput {
    pub optimization_sample_count: u32,
    pub validation_sample_count: u32,
    pub optimization_seed: u64,
    pub validation_seed: u64,
    pub marginals: Vec<QeconRobustMarginal>,
    pub latent_gaussian_correlation: [[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT],
}

/// Evidence-carrying robust assessment request for one nominal candidate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconRobustAssessmentScenario {
    pub base_scenario: QeconScenario,
    pub target_q_econ: f64,
    /// Required conditional probability, e.g. 0.95.
    pub required_viability_probability: f64,
    /// Separate digest receipt for the marginal/correlation/seed specification.
    /// Syntax validation only; canonical hashing is external.
    pub uncertainty_input_digest_sha256: String,
    pub uncertainty_input: QeconRobustUncertaintyInput,
    pub uncertainty_assumptions: Vec<String>,
}

/// Conditional robust-viability statistics from one seeded ensemble.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconRobustRunEstimate {
    pub seed: u64,
    pub sample_count: u32,
    pub model_evaluation_count: u64,
    pub q_econ: QeconScalarStatistics,
    pub utilization_fraction: QeconScalarStatistics,
    pub net_economic_rate_musd_per_m2_year: QeconScalarStatistics,
    pub probability_q_econ_at_least_target: QeconProbabilityEstimate,
}

/// Evidence-preserving robust assessment result.
///
/// A validation-qualified result means only that the independent validation
/// ensemble's Wilson lower bound meets the target probability under the
/// declared joint distribution. It is not physical qualification or an
/// unconditional real-plant probability forecast.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconRobustAssessmentResult {
    pub scenario_id: String,
    pub model_revision: String,
    pub robust_assessment_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub base_input_digest_sha256: String,
    pub uncertainty_input_digest_sha256: String,
    pub target_q_econ: f64,
    pub required_viability_probability: f64,
    pub optimization: QeconRobustRunEstimate,
    pub validation: QeconRobustRunEstimate,
    pub validation_qualified: bool,
    pub base_assumptions: Vec<String>,
    pub uncertainty_assumptions: Vec<String>,
    pub interpretation_limits: Vec<String>,
}

/// Evaluate one nominal candidate with separate optimization and validation
/// ensembles using the same declared joint uncertainty model.
///
/// This function evaluates a single candidate; it does not itself optimize
/// design variables. Reuse the optimization seed across candidate comparisons
/// to obtain common random numbers, but never reuse it for validation.
pub fn assess_qecon_robust_viability(
    scenario: &QeconRobustAssessmentScenario,
) -> Result<QeconRobustAssessmentResult, String> {
    let base: QeconScenarioResult = calculate_qecon_scenario(&scenario.base_scenario)?;
    validate_robust_assessment_scenario(scenario)?;

    let optimization = run_robust_ensemble(
        &scenario.base_scenario.input,
        scenario.target_q_econ,
        &scenario.uncertainty_input,
        scenario.uncertainty_input.optimization_seed,
        scenario.uncertainty_input.optimization_sample_count,
    )?;
    let validation = run_robust_ensemble(
        &scenario.base_scenario.input,
        scenario.target_q_econ,
        &scenario.uncertainty_input,
        scenario.uncertainty_input.validation_seed,
        scenario.uncertainty_input.validation_sample_count,
    )?;

    let validation_qualified = validation.probability_q_econ_at_least_target.wilson_lower_95
        >= scenario.required_viability_probability;

    Ok(QeconRobustAssessmentResult {
        scenario_id: base.scenario_id,
        model_revision: base.model_revision,
        robust_assessment_revision: QECON_ROBUST_ASSESSMENT_REVISION.to_string(),
        evidence_class: base.evidence_class,
        source_locator: base.source_locator,
        base_input_digest_sha256: base.input_digest_sha256,
        uncertainty_input_digest_sha256: scenario.uncertainty_input_digest_sha256.clone(),
        target_q_econ: scenario.target_q_econ,
        required_viability_probability: scenario.required_viability_probability,
        optimization,
        validation,
        validation_qualified,
        base_assumptions: base.assumptions,
        uncertainty_assumptions: scenario.uncertainty_assumptions.clone(),
        interpretation_limits: vec![
            "Optimization and validation ensembles use distinct deterministic seeds; validation is not used to select the candidate.".to_string(),
            "The same optimization seed should be reused across candidate designs to obtain common random numbers.".to_string(),
            "Wilson 95% intervals quantify finite Monte Carlo sampling error conditional on the supplied distribution only.".to_string(),
            "Marginal distributions and latent dependence are caller-supplied assumptions, not calibrated physical probability models.".to_string(),
            "The result is a conditional robust-viability assessment of one candidate, not a chance-constrained design optimizer.".to_string(),
            "It does not include model discrepancy, omitted physics, specialist heat-exhaust limits or material/maintenance qualification.".to_string(),
            "The base evidence class is inherited; Monte Carlo sampling never upgrades it.".to_string(),
        ],
    })
}

fn validate_robust_assessment_scenario(
    scenario: &QeconRobustAssessmentScenario,
) -> Result<(), String> {
    if !scenario.target_q_econ.is_finite() || scenario.target_q_econ <= 0.0 {
        return Err("target_q_econ must be finite and positive".to_string());
    }
    if !scenario.required_viability_probability.is_finite()
        || !(0.0..1.0).contains(&scenario.required_viability_probability)
    {
        return Err("required_viability_probability must be finite and in (0, 1)".to_string());
    }
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
        return Err("uncertainty assumptions must be non-empty".to_string());
    }
    let uncertainty = &scenario.uncertainty_input;
    if uncertainty.optimization_seed == uncertainty.validation_seed {
        return Err("optimization and validation seeds must be different".to_string());
    }
    for (name, count) in [
        ("optimization_sample_count", uncertainty.optimization_sample_count),
        ("validation_sample_count", uncertainty.validation_sample_count),
    ] {
        if !(2..=MAX_QECON_UNCERTAINTY_SAMPLES).contains(&count) {
            return Err(format!(
                "{name} must be in 2..={MAX_QECON_UNCERTAINTY_SAMPLES}"
            ));
        }
    }
    if uncertainty.marginals.len() != QECON_PARAMETER_COUNT {
        return Err(format!(
            "exactly {QECON_PARAMETER_COUNT} robust uncertainty marginals are required"
        ));
    }
    let mut seen = [false; QECON_PARAMETER_COUNT];
    for marginal in &uncertainty.marginals {
        marginal.validate()?;
        let index = marginal.parameter.index();
        if seen[index] {
            return Err(format!(
                "duplicate robust marginal for {:?}",
                marginal.parameter
            ));
        }
        seen[index] = true;
        // Check the extreme perturbations against the Q_econ domain. Since
        // the triangular marginal transform is monotone, all interior values
        // remain within the endpoint interval.
        for value in [marginal.minimum, marginal.maximum] {
            let transformed = apply_robust_value(&scenario.base_scenario.input, marginal, value)?;
            calculate_qecon(&transformed)?;
        }
    }
    if seen.iter().any(|present| !present) {
        return Err("all ten Q_econ parameters need one robust uncertainty marginal".to_string());
    }
    cholesky_correlation(&uncertainty.latent_gaussian_correlation)?;
    Ok(())
}

fn apply_robust_value(
    base: &QeconInput,
    marginal: &QeconRobustMarginal,
    sampled_value: f64,
) -> Result<QeconInput, String> {
    let nominal = marginal.parameter.value(base);
    let transformed = match marginal.transform {
        QeconRobustTransform::Multiplicative => nominal * sampled_value,
        QeconRobustTransform::Additive => nominal + sampled_value,
    };
    let transformed = if marginal.parameter == QeconParameter::PlantLifetime {
        transformed.round()
    } else {
        transformed
    };
    marginal.parameter.with_value(base, transformed)
}

fn run_robust_ensemble(
    base_input: &QeconInput,
    target_q_econ: f64,
    uncertainty: &QeconRobustUncertaintyInput,
    seed: u64,
    sample_count: u32,
) -> Result<QeconRobustRunEstimate, String> {
    let cholesky = cholesky_correlation(&uncertainty.latent_gaussian_correlation)?;
    let ordered_marginals = order_robust_marginals(&uncertainty.marginals)?;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut q_values = Vec::with_capacity(sample_count as usize);
    let mut utilization_values = Vec::with_capacity(sample_count as usize);
    let mut net_rate_values = Vec::with_capacity(sample_count as usize);
    let mut successes = 0_u32;

    for sample_index in 0..sample_count {
        let independent_normals = standard_normal_vector(&mut rng);
        let correlated_normals = multiply_lower_triangular(&cholesky, &independent_normals);
        let mut sampled_input = base_input.clone();

        for parameter in QeconParameter::ALL {
            let marginal = &ordered_marginals[parameter.index()];
            let latent = correlated_normals[parameter.index()];
            let uniform = standard_normal_cdf(latent).clamp(1e-12, 1.0 - 1e-12);
            let sampled_uncertainty = marginal.inverse_cdf(uniform);
            sampled_input = apply_robust_value(&sampled_input, marginal, sampled_uncertainty)
                .map_err(|error| format!("robust sample {sample_index}: {error}"))?;
        }
        let result = calculate_qecon(&sampled_input)
            .map_err(|error| format!("robust sample {sample_index}: {error}"))?;
        if result.q_econ >= target_q_econ {
            successes += 1;
        }
        q_values.push(result.q_econ);
        utilization_values.push(result.utilization_fraction);
        net_rate_values.push(result.net_economic_rate_musd_per_m2_year);
    }

    Ok(QeconRobustRunEstimate {
        seed,
        sample_count,
        model_evaluation_count: u64::from(sample_count),
        q_econ: scalar_statistics(&q_values)?,
        utilization_fraction: scalar_statistics(&utilization_values)?,
        net_economic_rate_musd_per_m2_year: scalar_statistics(&net_rate_values)?,
        probability_q_econ_at_least_target: wilson_probability_estimate(successes, sample_count),
    })
}

fn order_robust_marginals(
    marginals: &[QeconRobustMarginal],
) -> Result<[QeconRobustMarginal; QECON_PARAMETER_COUNT], String> {
    let mut ordered: [Option<QeconRobustMarginal>; QECON_PARAMETER_COUNT] =
        std::array::from_fn(|_| None);
    for marginal in marginals {
        ordered[marginal.parameter.index()] = Some(marginal.clone());
    }
    let mut values = Vec::with_capacity(QECON_PARAMETER_COUNT);
    for marginal in ordered {
        values.push(marginal.ok_or_else(|| "missing robust Q_econ marginal".to_string())?);
    }
    values
        .try_into()
        .map_err(|_| "expected ten robust Q_econ marginals".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qecon::{QECON_MODEL_REVISION, QeconDesignParameters};

    fn base_input() -> QeconInput {
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

    fn marginals() -> Vec<QeconRobustMarginal> {
        vec![
            QeconRobustMarginal { parameter: QeconParameter::FusionPowerDensity, transform: QeconRobustTransform::Multiplicative, minimum: 0.9, mode: 1.0, maximum: 1.1 },
            QeconRobustMarginal { parameter: QeconParameter::SurfaceReplacementTime, transform: QeconRobustTransform::Multiplicative, minimum: 0.8, mode: 1.0, maximum: 1.2 },
            QeconRobustMarginal { parameter: QeconParameter::SurfaceEnergyFluenceLimit, transform: QeconRobustTransform::Multiplicative, minimum: 0.9, mode: 1.0, maximum: 1.1 },
            QeconRobustMarginal { parameter: QeconParameter::NetEnergyConversionEfficiency, transform: QeconRobustTransform::Multiplicative, minimum: 0.9, mode: 1.0, maximum: 1.1 },
            QeconRobustMarginal { parameter: QeconParameter::NetPriceOfEnergy, transform: QeconRobustTransform::Multiplicative, minimum: 0.8, mode: 1.0, maximum: 1.2 },
            QeconRobustMarginal { parameter: QeconParameter::TargetCostPerEnergyYield, transform: QeconRobustTransform::Additive, minimum: 0.0, mode: 0.0, maximum: 0.001 },
            QeconRobustMarginal { parameter: QeconParameter::SurfaceReplacementCost, transform: QeconRobustTransform::Multiplicative, minimum: 0.8, mode: 1.0, maximum: 1.2 },
            QeconRobustMarginal { parameter: QeconParameter::PlantArealCost, transform: QeconRobustTransform::Multiplicative, minimum: 0.8, mode: 1.0, maximum: 1.2 },
            QeconRobustMarginal { parameter: QeconParameter::RealInterestRate, transform: QeconRobustTransform::Additive, minimum: -0.5, mode: 0.0, maximum: 0.5 },
            QeconRobustMarginal { parameter: QeconParameter::PlantLifetime, transform: QeconRobustTransform::Additive, minimum: -2.0, mode: 0.0, maximum: 2.0 },
        ]
    }

    fn identity_correlation() -> [[f64; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT] {
        let mut matrix = [[0.0; QECON_PARAMETER_COUNT]; QECON_PARAMETER_COUNT];
        for (index, row) in matrix.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        matrix
    }

    fn scenario() -> QeconRobustAssessmentScenario {
        QeconRobustAssessmentScenario {
            base_scenario: QeconScenario {
                scenario_id: "robust-qecon-fixture".to_string(),
                model_revision: QECON_MODEL_REVISION.to_string(),
                evidence_class: QeconEvidenceClass::ScenarioAssumption,
                source_locator: None,
                input_digest_sha256: "a".repeat(64),
                assumptions: vec!["Illustrative nominal plant design".to_string()],
                input: base_input(),
            },
            target_q_econ: 1.0,
            required_viability_probability: 0.9,
            uncertainty_input_digest_sha256: "b".repeat(64),
            uncertainty_input: QeconRobustUncertaintyInput {
                optimization_sample_count: 512,
                validation_sample_count: 2048,
                optimization_seed: 17,
                validation_seed: 29,
                marginals: marginals(),
                latent_gaussian_correlation: identity_correlation(),
            },
            uncertainty_assumptions: vec![
                "Triangular perturbations are illustrative scenario assumptions.".to_string(),
                "Latent correlation is identity for this test fixture.".to_string(),
            ],
        }
    }

    #[test]
    fn independent_optimization_and_validation_ensembles_are_reproducible() {
        let request = scenario();
        let first = assess_qecon_robust_viability(&request).unwrap();
        let second = assess_qecon_robust_viability(&request).unwrap();
        assert_eq!(first, second);
        assert_ne!(first.optimization.seed, first.validation.seed);
        assert_eq!(first.optimization.sample_count, 512);
        assert_eq!(first.validation.sample_count, 2048);
        assert_eq!(first.evidence_class, QeconEvidenceClass::ScenarioAssumption);
        assert_eq!(first.base_input_digest_sha256, "a".repeat(64));
        assert_eq!(first.uncertainty_input_digest_sha256, "b".repeat(64));
    }

    #[test]
    fn robust_assessment_has_distinct_sampling_intervals_and_model_assumptions() {
        let result = assess_qecon_robust_viability(&scenario()).unwrap();
        assert!(
            result.optimization.probability_q_econ_at_least_target.wilson_lower_95
                <= result.optimization.probability_q_econ_at_least_target.probability_estimate
        );
        assert!(
            result.validation.probability_q_econ_at_least_target.probability_estimate
                <= result.validation.probability_q_econ_at_least_target.wilson_upper_95
        );
        assert!(result.interpretation_limits.len() >= 5);
    }

    #[test]
    fn robust_assessment_rejects_reused_seeds_and_invalid_joint_inputs() {
        let mut request = scenario();
        request.uncertainty_input.validation_seed = request.uncertainty_input.optimization_seed;
        assert!(assess_qecon_robust_viability(&request).is_err());

        request = scenario();
        request.uncertainty_input.latent_gaussian_correlation[0][1] = 0.8;
        assert!(assess_qecon_robust_viability(&request).is_err());

        request = scenario();
        request.uncertainty_input.marginals.pop();
        assert!(assess_qecon_robust_viability(&request).is_err());
    }

    #[test]
    fn robust_assessment_detects_uncertainty_that_straddles_economic_viability() {
        let mut request = scenario();
        request.target_q_econ = 1.5;
        request.required_viability_probability = 0.95;
        request.uncertainty_input.optimization_sample_count = 2048;
        request.uncertainty_input.validation_sample_count = 4096;
        request.base_scenario.input.fusion_power_density_mw_per_m2 = 2.8;
        let result = assess_qecon_robust_viability(&request).unwrap();
        assert!(result.validation.probability_q_econ_at_least_target.probability_estimate < 1.0);
    }
}
