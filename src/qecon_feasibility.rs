//! Q_econ reachability inside an externally declared engineering power-density envelope.
//!
//! This module does not calculate heat-exhaust, plasma-stability or materials
//! limits. It consumes a caller-supplied power-density interval and preserves
//! the evidence class/source for that interval. Its status values mean only
//! that a Q_econ threshold is or is not reachable *within the declared
//! interval*. They do not imply physical qualification of the underlying
//! limit.

use crate::qecon::{
    QeconAsymptoticLimits, QeconEvidenceClass, QeconInput, QeconScenario,
    QeconScenarioResult, calculate_qecon, calculate_qecon_asymptotic_limits,
    calculate_qecon_scenario, required_fusion_power_density_for_qecon,
};
use serde::{Deserialize, Serialize};

pub const QECON_FEASIBILITY_REVISION: &str = "qecon-feasibility-envelope-v0.1";

/// A caller-supplied engineering interval for fusion power density.
///
/// The interval must use the same declared control surface as the Q_econ
/// scenario. The two identifiers are carried separately and must match
/// exactly. This module validates units-by-contract and identity strings; it
/// cannot independently establish that the upstream thermal/material model
/// really uses that boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconPowerDensityEnvelope {
    pub envelope_control_surface_id: String,
    pub minimum_power_density_mw_per_m2: f64,
    pub maximum_power_density_mw_per_m2: f64,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub model_revision: String,
    /// SHA-256 receipt for the external limit inputs; syntax checked only.
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
}

/// Full evaluation request, including the base scenario's declared surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconFeasibilityScenario {
    pub base_scenario: QeconScenario,
    pub scenario_control_surface_id: String,
    pub target_q_econ: f64,
    pub envelope: QeconPowerDensityEnvelope,
}

/// Mathematical reachability classification inside the supplied interval.
///
/// These labels are deliberately not named "physically feasible": they
/// describe the relationship between the Q_econ curve and an external
/// engineering interval whose qualification remains the caller's task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconEnvelopeStatus {
    /// The requested target is at or above a finite Q_econ asymptotic ceiling.
    EconomicallyAsymptoticallyUnreachable,
    /// The target has a finite mathematical root, but it exceeds the declared
    /// maximum power-density bound.
    AboveDeclaredMaximum,
    /// The target is already achieved at the declared minimum power density.
    TargetMetAtMinimum,
    /// The threshold-crossing density lies within the declared interval.
    WithinDeclaredInterval,
}

/// Evidence-preserving result of checking a Q_econ target against a declared
/// power-density envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconFeasibilityResult {
    pub scenario_id: String,
    pub feasibility_model_revision: String,
    pub qecon_model_revision: String,
    pub base_evidence_class: QeconEvidenceClass,
    pub base_source_locator: Option<String>,
    pub base_input_digest_sha256: String,
    pub envelope_evidence_class: QeconEvidenceClass,
    pub envelope_source_locator: Option<String>,
    pub envelope_model_revision: String,
    pub envelope_input_digest_sha256: String,
    pub control_surface_id: String,
    pub target_q_econ: f64,
    pub asymptotic_limits: QeconAsymptoticLimits,
    pub required_power_density_mw_per_m2: Option<f64>,
    pub minimum_power_density_mw_per_m2: f64,
    pub maximum_power_density_mw_per_m2: f64,
    pub q_econ_at_minimum_density: f64,
    pub q_econ_at_maximum_density: f64,
    /// Maximum declared power density less required density. Negative values
    /// indicate that the economic target lies above the declared interval.
    pub margin_to_maximum_mw_per_m2: Option<f64>,
    pub status: QeconEnvelopeStatus,
    pub base_assumptions: Vec<String>,
    pub envelope_assumptions: Vec<String>,
    pub interpretation_limits: Vec<String>,
}

/// Evaluate a Q_econ threshold against an externally supplied power-density
/// interval.
///
/// The envelope must use exactly the same control-surface identifier as the
/// scenario. It must carry its own source/evidence/revision/digest metadata.
/// Digests are syntax-checked only; this crate does not perform canonical
/// serialization or cryptographic verification.
pub fn evaluate_qecon_power_density_envelope(
    scenario: &QeconFeasibilityScenario,
) -> Result<QeconFeasibilityResult, String> {
    let base: QeconScenarioResult = calculate_qecon_scenario(&scenario.base_scenario)?;
    validate_envelope(scenario)?;

    let minimum_input = with_power_density(
        &scenario.base_scenario.input,
        scenario.envelope.minimum_power_density_mw_per_m2,
    );
    let maximum_input = with_power_density(
        &scenario.base_scenario.input,
        scenario.envelope.maximum_power_density_mw_per_m2,
    );
    let q_min = calculate_qecon(&minimum_input)?.q_econ;
    let q_max = calculate_qecon(&maximum_input)?.q_econ;
    let asymptotic_limits =
        calculate_qecon_asymptotic_limits(&scenario.base_scenario.input.design)?;
    let required_power_density_mw_per_m2 = required_fusion_power_density_for_qecon(
        &scenario.base_scenario.input.design,
        scenario.target_q_econ,
    )
    .ok();

    let status = match required_power_density_mw_per_m2 {
        None => QeconEnvelopeStatus::EconomicallyAsymptoticallyUnreachable,
        Some(required) if required > scenario.envelope.maximum_power_density_mw_per_m2 => {
            QeconEnvelopeStatus::AboveDeclaredMaximum
        }
        Some(required)
            if required <= scenario.envelope.minimum_power_density_mw_per_m2
                || q_min >= scenario.target_q_econ =>
        {
            QeconEnvelopeStatus::TargetMetAtMinimum
        }
        Some(_) => QeconEnvelopeStatus::WithinDeclaredInterval,
    };

    let margin_to_maximum_mw_per_m2 = required_power_density_mw_per_m2
        .map(|required| scenario.envelope.maximum_power_density_mw_per_m2 - required);

    Ok(QeconFeasibilityResult {
        scenario_id: base.scenario_id,
        feasibility_model_revision: QECON_FEASIBILITY_REVISION.to_string(),
        qecon_model_revision: base.model_revision,
        base_evidence_class: base.evidence_class,
        base_source_locator: base.source_locator,
        base_input_digest_sha256: base.input_digest_sha256,
        envelope_evidence_class: scenario.envelope.evidence_class,
        envelope_source_locator: scenario.envelope.source_locator.clone(),
        envelope_model_revision: scenario.envelope.model_revision.clone(),
        envelope_input_digest_sha256: scenario.envelope.input_digest_sha256.clone(),
        control_surface_id: scenario.scenario_control_surface_id.clone(),
        target_q_econ: scenario.target_q_econ,
        asymptotic_limits,
        required_power_density_mw_per_m2,
        minimum_power_density_mw_per_m2: scenario
            .envelope
            .minimum_power_density_mw_per_m2,
        maximum_power_density_mw_per_m2: scenario
            .envelope
            .maximum_power_density_mw_per_m2,
        q_econ_at_minimum_density: q_min,
        q_econ_at_maximum_density: q_max,
        margin_to_maximum_mw_per_m2,
        status,
        base_assumptions: base.assumptions,
        envelope_assumptions: scenario.envelope.assumptions.clone(),
        interpretation_limits: vec![
            "The interval is caller-supplied and has not been independently qualified by this module.".to_string(),
            "Within-declared-interval means the Q_econ threshold lies within the supplied interval; it is not proof of physical feasibility.".to_string(),
            "Heat exhaust, plasma stability, neutron loading, material damage and thermal-hydraulic limits require specialist validated models.".to_string(),
            "The envelope and base scenario must refer to the same physical control surface and compatible power averaging/timebase.".to_string(),
            "SHA-256 values are carried and syntax-checked only; canonical digest verification belongs to the shared evidence layer.".to_string(),
        ],
    })
}

fn validate_envelope(scenario: &QeconFeasibilityScenario) -> Result<(), String> {
    if scenario.scenario_control_surface_id.trim().is_empty()
        || scenario
            .envelope
            .envelope_control_surface_id
            .trim()
            .is_empty()
    {
        return Err("both control-surface identifiers must be non-empty".to_string());
    }
    if scenario.scenario_control_surface_id != scenario.envelope.envelope_control_surface_id {
        return Err(
            "Q_econ scenario and engineering envelope control-surface identifiers must match"
                .to_string(),
        );
    }
    if !scenario.target_q_econ.is_finite() || scenario.target_q_econ <= 0.0 {
        return Err("target_q_econ must be finite and positive".to_string());
    }
    let minimum = scenario.envelope.minimum_power_density_mw_per_m2;
    let maximum = scenario.envelope.maximum_power_density_mw_per_m2;
    if !minimum.is_finite() || !maximum.is_finite() || minimum <= 0.0 || maximum < minimum {
        return Err(
            "power-density envelope must satisfy finite 0 < minimum <= maximum".to_string(),
        );
    }
    if scenario.envelope.model_revision.trim().is_empty() {
        return Err("envelope model_revision must not be empty".to_string());
    }
    validate_sha256(
        &scenario.envelope.input_digest_sha256,
        "envelope input digest",
    )?;
    if scenario.envelope.assumptions.is_empty()
        || scenario
            .envelope
            .assumptions
            .iter()
            .any(|assumption| assumption.trim().is_empty())
    {
        return Err("engineering envelope requires non-empty assumptions".to_string());
    }
    if matches!(
        scenario.envelope.evidence_class,
        QeconEvidenceClass::Measured
            | QeconEvidenceClass::LiteratureReported
            | QeconEvidenceClass::EngineeringEstimate
    ) && scenario
        .envelope
        .source_locator
        .as_ref()
        .is_none_or(|source| source.trim().is_empty())
    {
        return Err(
            "measured, literature and engineering envelopes require a source locator".to_string(),
        );
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "{label} must contain exactly 64 hexadecimal characters"
        ));
    }
    Ok(())
}

fn with_power_density(input: &QeconInput, power_density: f64) -> QeconInput {
    let mut result = input.clone();
    result.fusion_power_density_mw_per_m2 = power_density;
    result
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

    fn scenario(target: f64, minimum: f64, maximum: f64) -> QeconFeasibilityScenario {
        QeconFeasibilityScenario {
            base_scenario: QeconScenario {
                scenario_id: "qecon-envelope-test".to_string(),
                model_revision: QECON_MODEL_REVISION.to_string(),
                evidence_class: QeconEvidenceClass::ScenarioAssumption,
                source_locator: None,
                input_digest_sha256: "a".repeat(64),
                assumptions: vec!["Published illustrative Q_econ design point".to_string()],
                input: base_input(),
            },
            scenario_control_surface_id: "fusion-extraction-surface-S".to_string(),
            target_q_econ: target,
            envelope: QeconPowerDensityEnvelope {
                envelope_control_surface_id: "fusion-extraction-surface-S".to_string(),
                minimum_power_density_mw_per_m2: minimum,
                maximum_power_density_mw_per_m2: maximum,
                evidence_class: QeconEvidenceClass::EngineeringEstimate,
                source_locator: Some("https://example.org/thermal-limit-model".to_string()),
                model_revision: "thermal-envelope-model-v1".to_string(),
                input_digest_sha256: "b".repeat(64),
                assumptions: vec![
                    "Illustrative evidence-backed upper density bound".to_string(),
                    "Power density uses the same declared extraction surface".to_string(),
                ],
            },
        }
    }

    #[test]
    fn reports_target_within_declared_density_interval() {
        let result = evaluate_qecon_power_density_envelope(&scenario(1.5, 1.0, 5.0)).unwrap();
        assert_eq!(result.status, QeconEnvelopeStatus::WithinDeclaredInterval);
        assert!(result.required_power_density_mw_per_m2.unwrap() > 1.0);
        assert!(result.required_power_density_mw_per_m2.unwrap() < 5.0);
        assert!(result.margin_to_maximum_mw_per_m2.unwrap() > 0.0);
    }

    #[test]
    fn reports_target_above_declared_density_ceiling() {
        let result = evaluate_qecon_power_density_envelope(&scenario(1.5, 1.0, 2.0)).unwrap();
        assert_eq!(result.status, QeconEnvelopeStatus::AboveDeclaredMaximum);
        assert!(result.margin_to_maximum_mw_per_m2.unwrap() < 0.0);
    }

    #[test]
    fn reports_target_already_met_at_lower_density_bound() {
        let result = evaluate_qecon_power_density_envelope(&scenario(1.0, 3.0, 5.0)).unwrap();
        assert_eq!(result.status, QeconEnvelopeStatus::TargetMetAtMinimum);
        assert!(result.q_econ_at_minimum_density >= 1.0);
    }

    #[test]
    fn reports_asymptotically_unreachable_target() {
        let design = base_input().design;
        let limit = calculate_qecon_asymptotic_limits(&design)
            .unwrap()
            .maximum_q_econ
            .unwrap();
        let result =
            evaluate_qecon_power_density_envelope(&scenario(limit * 1.01, 1.0, 100.0)).unwrap();
        assert_eq!(
            result.status,
            QeconEnvelopeStatus::EconomicallyAsymptoticallyUnreachable
        );
        assert_eq!(result.required_power_density_mw_per_m2, None);
    }

    #[test]
    fn rejects_mismatched_control_surface_and_invalid_provenance() {
        let mut request = scenario(1.5, 1.0, 5.0);
        request.envelope.envelope_control_surface_id = "different-surface".to_string();
        assert!(evaluate_qecon_power_density_envelope(&request).is_err());

        request = scenario(1.5, 1.0, 5.0);
        request.envelope.input_digest_sha256 = "invalid".to_string();
        assert!(evaluate_qecon_power_density_envelope(&request).is_err());
    }

    #[test]
    fn preserves_distinct_base_and_envelope_evidence_classes() {
        let result = evaluate_qecon_power_density_envelope(&scenario(1.5, 1.0, 5.0)).unwrap();
        assert_eq!(
            result.base_evidence_class,
            QeconEvidenceClass::ScenarioAssumption
        );
        assert_eq!(
            result.envelope_evidence_class,
            QeconEvidenceClass::EngineeringEstimate
        );
        assert!(result.interpretation_limits.len() >= 4);
    }
}
