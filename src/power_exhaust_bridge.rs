//! Conservation-ledger bridge from Q_econ extraction power to divertor target loads.
//!
//! This module is an accounting and evidence-boundary adapter. It does not
//! solve the plasma edge, scrape-off layer, divertor neutrals, detachment,
//! transient transport, thermal hydraulics or material response. Those
//! quantities must come from separately identified upstream models or
//! measurements and are carried here as explicit inputs.
//!
//! Most importantly, Q_econ's fusion power density is normalized to an energy
//! extraction surface. Divertor target heat flux is normalized to the target's
//! wetted area. The areas are not interchangeable. This bridge only converts
//! between them when the extraction area, power ledger, target shares, target
//! wetted areas and provenance are all explicit.

use crate::qecon::{
    QeconEvidenceClass, QeconResult, QeconScenario, calculate_qecon_scenario,
};
use serde::{Deserialize, Serialize};

pub const POWER_EXHAUST_BRIDGE_REVISION: &str = "power-exhaust-boundary-bridge-v0.1";

/// Semantic role of an area used in the power-exhaust calculation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerSurfaceRole {
    /// Q_econ fusion power density is normalized to this energy-extraction surface.
    EnergyExtraction,
    /// Divertor target heat flux is normalized to this target's wetted area.
    DivertorWettedTarget,
}

/// Area evidence with an explicit physical role and independent provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerSurfaceArea {
    pub surface_id: String,
    pub role: PowerSurfaceRole,
    pub area_m2: f64,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub model_revision: String,
    /// SHA-256 receipt; this bridge validates syntax, not canonical hashing.
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
}

/// Temporal basis shared by every power term in one ledger.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerTimeBasisKind {
    /// A steady operating-point balance. No averaging window is supplied.
    SteadyStateAverage,
    /// A finite-window average. The interval length must be explicit.
    WindowAverage,
}

/// The time basis must be common to every power term and target-load estimate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerTimeBasis {
    pub basis_id: String,
    pub kind: PowerTimeBasisKind,
    pub averaging_window_seconds: Option<f64>,
}

/// Explicit inputs to the confined-plasma / separatrix / SOL power ledger.
///
/// All power values are MW. The alpha fraction is the effective fraction of
/// total fusion power deposited as plasma alpha heating for the declared
/// reaction mix and confinement assumptions; it is deliberately not hardcoded
/// to a D-T value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExhaustPowerBalance {
    /// Alpha heating deposited in the confined plasma divided by total fusion power.
    pub effective_alpha_heating_fraction_of_fusion_power: f64,
    /// Total non-alpha heating deposited in the confined plasma, MW.
    pub auxiliary_heating_power_mw: f64,
    /// Radiation emitted from the confined-plasma/core boundary, MW.
    pub core_radiated_power_mw: f64,
    /// dW/dt for stored plasma energy, MW; positive means energy is accumulating.
    pub stored_energy_change_rate_mw: f64,
    /// Fraction of separatrix-crossing power radiated in the SOL/divertor region.
    pub sol_radiated_fraction: f64,
    /// Fraction of post-SOL-radiation power routed to the modelled divertor targets.
    pub fraction_of_post_sol_power_routed_to_divertor: f64,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub model_revision: String,
    /// SHA-256 receipt; this bridge validates syntax, not canonical hashing.
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
}

/// One explicitly modelled divertor target region.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DivertorTargetInput {
    pub target_id: String,
    pub wetted_surface: PowerSurfaceArea,
    /// Share of total divertor-routed power deposited on this target.
    /// Shares across all listed targets must sum to one.
    pub share_of_divertor_power: f64,
    /// Peak-to-average heat-flux factor supplied by an upstream edge/geometry model.
    /// It must be >= 1 and is not estimated by this bridge.
    pub peak_to_average_factor: f64,
    pub assumptions: Vec<String>,
}

/// Complete request. The Q_econ surface identity is repeated explicitly so
/// that callers cannot silently join a density to an unrelated area.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerExhaustBridgeInput {
    pub qecon_scenario: QeconScenario,
    pub qecon_control_surface_id: String,
    pub extraction_surface: PowerSurfaceArea,
    pub time_basis: PowerTimeBasis,
    pub power_balance: ExhaustPowerBalance,
    pub divertor_targets: Vec<DivertorTargetInput>,
}

/// Result for one divertor target, retaining its own geometry evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DivertorTargetLoad {
    pub target_id: String,
    pub surface_id: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub model_revision: String,
    pub input_digest_sha256: String,
    pub share_of_divertor_power: f64,
    pub wetted_area_m2: f64,
    pub target_power_mw: f64,
    pub average_heat_flux_mw_per_m2: f64,
    pub peak_to_average_factor: f64,
    pub estimated_peak_heat_flux_mw_per_m2: f64,
    pub assumptions: Vec<String>,
}

/// Conservation-accounted mapping result.
///
/// This is a boundary-bridge calculation only. It is not a validated
/// divertor-physics prediction or a statement that the resulting target load
/// is within an engineering material limit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerExhaustBridgeResult {
    pub bridge_model_revision: String,
    pub qecon_scenario_id: String,
    pub qecon_model_revision: String,
    pub qecon_evidence_class: QeconEvidenceClass,
    pub qecon_input_digest_sha256: String,
    pub qecon_result: QeconResult,
    pub qecon_control_surface_id: String,
    pub extraction_surface: PowerSurfaceArea,
    pub time_basis: PowerTimeBasis,
    pub power_balance_evidence_class: QeconEvidenceClass,
    pub power_balance_source_locator: Option<String>,
    pub power_balance_model_revision: String,
    pub power_balance_input_digest_sha256: String,
    pub fusion_power_density_mw_per_m2: f64,
    pub total_fusion_power_mw: f64,
    pub alpha_heating_power_mw: f64,
    pub auxiliary_heating_power_mw: f64,
    pub core_radiated_power_mw: f64,
    pub stored_energy_change_rate_mw: f64,
    pub separatrix_crossing_power_mw: f64,
    pub sol_radiated_power_mw: f64,
    pub post_sol_power_mw: f64,
    pub divertor_routed_power_mw: f64,
    pub other_post_sol_power_mw: f64,
    pub divertor_targets: Vec<DivertorTargetLoad>,
    pub assumptions: Vec<String>,
    pub interpretation_limits: Vec<String>,
}

/// Convert one evidence-carrying Q_econ scenario into target heat-load estimates.
///
/// Energy accounting:
///
/// - P_fusion = (P_fusion / S)_Qecon * A_extraction
/// - P_alpha = P_fusion * f_alpha, effective deposited fraction
/// - P_sep = P_alpha + P_aux - P_core-rad - dW/dt
/// - P_SOL = P_sep * (1 - f_SOL-rad)
/// - P_div = P_SOL * f_divertor
/// - P_target_i = P_div * share_i
/// - q_avg_i = P_target_i / A_wetted_i
/// - q_peak_i = q_avg_i * peak_to_average_i
///
/// The upstream solver or measurement chain owns f_alpha, radiation fractions,
/// divertor routing, target shares, wetted areas and peak factors. This method
/// checks their declared domains, conserves the supplied power ledger and
/// preserves their evidence; it does not establish their physical accuracy.
pub fn map_qecon_to_divertor_loads(
    input: &PowerExhaustBridgeInput,
) -> Result<PowerExhaustBridgeResult, String> {
    validate_input(input)?;
    let qecon = calculate_qecon_scenario(&input.qecon_scenario)?;

    let fusion_power_density = input
        .qecon_scenario
        .input
        .fusion_power_density_mw_per_m2;
    let total_fusion_power = fusion_power_density * input.extraction_surface.area_m2;
    let alpha_heating = total_fusion_power
        * input
            .power_balance
            .effective_alpha_heating_fraction_of_fusion_power;
    let separatrix_power = alpha_heating
        + input.power_balance.auxiliary_heating_power_mw
        - input.power_balance.core_radiated_power_mw
        - input.power_balance.stored_energy_change_rate_mw;
    let sol_radiated_power = separatrix_power * input.power_balance.sol_radiated_fraction;
    let post_sol_power = separatrix_power - sol_radiated_power;
    let divertor_power = post_sol_power
        * input
            .power_balance
            .fraction_of_post_sol_power_routed_to_divertor;
    let other_post_sol_power = post_sol_power - divertor_power;

    let divertor_targets = input
        .divertor_targets
        .iter()
        .map(|target| {
            let target_power = divertor_power * target.share_of_divertor_power;
            let average_heat_flux = target_power / target.wetted_surface.area_m2;
            let peak_heat_flux = average_heat_flux * target.peak_to_average_factor;
            DivertorTargetLoad {
                target_id: target.target_id.clone(),
                surface_id: target.wetted_surface.surface_id.clone(),
                evidence_class: target.wetted_surface.evidence_class,
                source_locator: target.wetted_surface.source_locator.clone(),
                model_revision: target.wetted_surface.model_revision.clone(),
                input_digest_sha256: target.wetted_surface.input_digest_sha256.clone(),
                share_of_divertor_power: target.share_of_divertor_power,
                wetted_area_m2: target.wetted_surface.area_m2,
                target_power_mw: target_power,
                average_heat_flux_mw_per_m2: average_heat_flux,
                peak_to_average_factor: target.peak_to_average_factor,
                estimated_peak_heat_flux_mw_per_m2: peak_heat_flux,
                assumptions: target.assumptions.clone(),
            }
        })
        .collect::<Vec<_>>();

    if divertor_targets.iter().any(|target| {
        !target.target_power_mw.is_finite()
            || !target.average_heat_flux_mw_per_m2.is_finite()
            || !target.estimated_peak_heat_flux_mw_per_m2.is_finite()
    }) {
        return Err("computed target power and heat-flux outputs must be finite".to_string());
    }

    Ok(PowerExhaustBridgeResult {
        bridge_model_revision: POWER_EXHAUST_BRIDGE_REVISION.to_string(),
        qecon_scenario_id: qecon.scenario_id,
        qecon_model_revision: qecon.model_revision,
        qecon_evidence_class: qecon.evidence_class,
        qecon_input_digest_sha256: qecon.input_digest_sha256,
        qecon_result: qecon.result,
        qecon_control_surface_id: input.qecon_control_surface_id.clone(),
        extraction_surface: input.extraction_surface.clone(),
        time_basis: input.time_basis.clone(),
        power_balance_evidence_class: input.power_balance.evidence_class,
        power_balance_source_locator: input.power_balance.source_locator.clone(),
        power_balance_model_revision: input.power_balance.model_revision.clone(),
        power_balance_input_digest_sha256: input.power_balance.input_digest_sha256.clone(),
        fusion_power_density_mw_per_m2: fusion_power_density,
        total_fusion_power_mw: total_fusion_power,
        alpha_heating_power_mw: alpha_heating,
        auxiliary_heating_power_mw: input.power_balance.auxiliary_heating_power_mw,
        core_radiated_power_mw: input.power_balance.core_radiated_power_mw,
        stored_energy_change_rate_mw: input.power_balance.stored_energy_change_rate_mw,
        separatrix_crossing_power_mw: separatrix_power,
        sol_radiated_power_mw: sol_radiated_power,
        post_sol_power_mw: post_sol_power,
        divertor_routed_power_mw: divertor_power,
        other_post_sol_power_mw: other_post_sol_power,
        divertor_targets,
        assumptions: input.power_balance.assumptions.clone(),
        interpretation_limits: vec![
            "Q_econ extraction-surface area and divertor target wetted area are distinct physical boundaries and are never substituted for one another.".to_string(),
            "Matching surface identifiers enforce a declared software join only; they do not independently prove that the supplied geometry or upstream model uses the same physical boundary.".to_string(),
            "The effective alpha-heating fraction, auxiliary heating, core radiation, stored-energy derivative, SOL radiation, divertor routing, wetted areas and peak factors are caller-supplied inputs from separately identified sources.".to_string(),
            "This bridge performs declared power accounting and area normalization only; it does not solve plasma-edge transport, detachment, neutral dynamics, transient control or target thermal response.".to_string(),
            "The peak-to-average factor is supplied by an upstream model or measurement; it is not inferred from the average target heat flux.".to_string(),
            "A computed heat flux is not proof of material survivability, component lifetime, safe operating margin or plant availability.".to_string(),
            "Evidence classes are preserved and never upgraded by calculation. SHA-256 receipts are syntax-checked only; canonical digest verification belongs to the shared evidence layer.".to_string(),
        ],
    })
}

fn validate_input(input: &PowerExhaustBridgeInput) -> Result<(), String> {
    if input.qecon_control_surface_id.trim().is_empty()
        || input.extraction_surface.surface_id.trim().is_empty()
    {
        return Err("Q_econ control-surface and extraction-surface identifiers must be non-empty".to_string());
    }
    if input.qecon_control_surface_id != input.extraction_surface.surface_id {
        return Err("Q_econ control-surface identifier must match the extraction-surface identifier".to_string());
    }
    validate_surface(&input.extraction_surface, PowerSurfaceRole::EnergyExtraction)?;
    validate_time_basis(&input.time_basis)?;
    validate_provenance(
        input.power_balance.evidence_class,
        input.power_balance.source_locator.as_deref(),
        &input.power_balance.model_revision,
        &input.power_balance.input_digest_sha256,
        "power-balance",
    )?;

    let balance = &input.power_balance;
    validate_fraction(
        balance.effective_alpha_heating_fraction_of_fusion_power,
        "effective_alpha_heating_fraction_of_fusion_power",
    )?;
    validate_nonnegative_finite(balance.auxiliary_heating_power_mw, "auxiliary_heating_power_mw")?;
    validate_nonnegative_finite(balance.core_radiated_power_mw, "core_radiated_power_mw")?;
    if !balance.stored_energy_change_rate_mw.is_finite() {
        return Err("stored_energy_change_rate_mw must be finite".to_string());
    }
    validate_fraction(balance.sol_radiated_fraction, "sol_radiated_fraction")?;
    validate_fraction(
        balance.fraction_of_post_sol_power_routed_to_divertor,
        "fraction_of_post_sol_power_routed_to_divertor",
    )?;

    let fusion_power = input.qecon_scenario.input.fusion_power_density_mw_per_m2
        * input.extraction_surface.area_m2;
    let alpha_heating = fusion_power * balance.effective_alpha_heating_fraction_of_fusion_power;
    let separatrix_power = alpha_heating
        + balance.auxiliary_heating_power_mw
        - balance.core_radiated_power_mw
        - balance.stored_energy_change_rate_mw;
    if !separatrix_power.is_finite() || separatrix_power < 0.0 {
        return Err("declared power ledger produces negative or non-finite separatrix power".to_string());
    }

    if input.divertor_targets.is_empty() {
        return Err("at least one divertor target must be declared".to_string());
    }
    let mut target_ids = std::collections::HashSet::new();
    let mut surface_ids = std::collections::HashSet::new();
    surface_ids.insert(input.extraction_surface.surface_id.as_str());
    let mut share_sum = 0.0;
    for target in &input.divertor_targets {
        if target.target_id.trim().is_empty() || !target_ids.insert(target.target_id.as_str()) {
            return Err("divertor target identifiers must be non-empty and unique".to_string());
        }
        validate_surface(&target.wetted_surface, PowerSurfaceRole::DivertorWettedTarget)?;
        if !surface_ids.insert(target.wetted_surface.surface_id.as_str()) {
            return Err("extraction and divertor target surface identifiers must be distinct and unique".to_string());
        }
        validate_fraction(target.share_of_divertor_power, "share_of_divertor_power")?;
        if target.share_of_divertor_power <= 0.0 {
            return Err("every declared divertor target must have a positive power share".to_string());
        }
        if !target.peak_to_average_factor.is_finite() || target.peak_to_average_factor < 1.0 {
            return Err("peak_to_average_factor must be finite and >= 1".to_string());
        }
        share_sum += target.share_of_divertor_power;
    }
    if (share_sum - 1.0).abs() > 1.0e-9 {
        return Err("divertor target power shares must sum to one; account for every routed target branch".to_string());
    }
    Ok(())
}

fn validate_surface(surface: &PowerSurfaceArea, expected_role: PowerSurfaceRole) -> Result<(), String> {
    if surface.surface_id.trim().is_empty() {
        return Err("surface_id must not be empty".to_string());
    }
    if surface.role != expected_role {
        return Err(format!("surface {} has an incompatible physical role", surface.surface_id));
    }
    if !surface.area_m2.is_finite() || surface.area_m2 <= 0.0 {
        return Err(format!("surface {} area_m2 must be finite and positive", surface.surface_id));
    }
    validate_provenance(
        surface.evidence_class,
        surface.source_locator.as_deref(),
        &surface.model_revision,
        &surface.input_digest_sha256,
        "surface-area",
    )
}

fn validate_time_basis(basis: &PowerTimeBasis) -> Result<(), String> {
    if basis.basis_id.trim().is_empty() {
        return Err("time-basis identifier must not be empty".to_string());
    }
    match basis.kind {
        PowerTimeBasisKind::SteadyStateAverage => {
            if basis.averaging_window_seconds.is_some() {
                return Err("steady-state time basis must not declare a finite averaging window".to_string());
            }
        }
        PowerTimeBasisKind::WindowAverage => {
            let window = basis.averaging_window_seconds.ok_or_else(|| {
                "window-average time basis requires averaging_window_seconds".to_string()
            })?;
            if !window.is_finite() || window <= 0.0 {
                return Err("averaging_window_seconds must be finite and positive".to_string());
            }
        }
    }
    Ok(())
}

fn validate_provenance(
    evidence_class: QeconEvidenceClass,
    source_locator: Option<&str>,
    model_revision: &str,
    digest: &str,
    label: &str,
) -> Result<(), String> {
    if model_revision.trim().is_empty() {
        return Err(format!("{label} model_revision must not be empty"));
    }
    if evidence_class != QeconEvidenceClass::ScenarioAssumption
        && evidence_class != QeconEvidenceClass::SyntheticFixture
        && source_locator.is_none_or(|source| source.trim().is_empty())
    {
        return Err(format!("{label} evidence requires a source locator"));
    }
    validate_sha256(digest, &format!("{label} input digest"))
}

fn validate_sha256(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{label} must contain exactly 64 hexadecimal characters"));
    }
    Ok(())
}

fn validate_fraction(value: f64, label: &str) -> Result<(), String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(format!("{label} must be finite and in [0, 1]"));
    }
    Ok(())
}

fn validate_nonnegative_finite(value: f64, label: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{label} must be finite and non-negative"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qecon::{QECON_MODEL_REVISION, QeconDesignParameters, QeconInput};

    fn base_scenario() -> QeconScenario {
        QeconScenario {
            scenario_id: "exhaust-bridge-test".to_string(),
            model_revision: QECON_MODEL_REVISION.to_string(),
            evidence_class: QeconEvidenceClass::ScenarioAssumption,
            source_locator: None,
            input_digest_sha256: "a".repeat(64),
            assumptions: vec!["Synthetic test scenario only".to_string()],
            input: QeconInput {
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
            },
        }
    }

    fn surface(id: &str, role: PowerSurfaceRole, area: f64) -> PowerSurfaceArea {
        PowerSurfaceArea {
            surface_id: id.to_string(),
            role,
            area_m2: area,
            evidence_class: QeconEvidenceClass::SyntheticFixture,
            source_locator: None,
            model_revision: "synthetic-geometry-v1".to_string(),
            input_digest_sha256: "b".repeat(64),
            assumptions: vec!["Synthetic geometry fixture".to_string()],
        }
    }

    fn request() -> PowerExhaustBridgeInput {
        PowerExhaustBridgeInput {
            qecon_scenario: base_scenario(),
            qecon_control_surface_id: "energy-capture-S".to_string(),
            extraction_surface: surface(
                "energy-capture-S",
                PowerSurfaceRole::EnergyExtraction,
                10.0,
            ),
            time_basis: PowerTimeBasis {
                basis_id: "steady-point-v1".to_string(),
                kind: PowerTimeBasisKind::SteadyStateAverage,
                averaging_window_seconds: None,
            },
            power_balance: ExhaustPowerBalance {
                effective_alpha_heating_fraction_of_fusion_power: 0.2,
                auxiliary_heating_power_mw: 3.6,
                core_radiated_power_mw: 2.0,
                stored_energy_change_rate_mw: 0.0,
                sol_radiated_fraction: 0.5,
                fraction_of_post_sol_power_routed_to_divertor: 0.8,
                evidence_class: QeconEvidenceClass::SyntheticFixture,
                source_locator: None,
                model_revision: "synthetic-power-balance-v1".to_string(),
                input_digest_sha256: "c".repeat(64),
                assumptions: vec!["Synthetic power-balance fixture".to_string()],
            },
            divertor_targets: vec![
                DivertorTargetInput {
                    target_id: "outer".to_string(),
                    wetted_surface: surface(
                        "outer-wetted",
                        PowerSurfaceRole::DivertorWettedTarget,
                        0.2,
                    ),
                    share_of_divertor_power: 0.5,
                    peak_to_average_factor: 1.5,
                    assumptions: vec!["Synthetic target model".to_string()],
                },
                DivertorTargetInput {
                    target_id: "inner".to_string(),
                    wetted_surface: surface(
                        "inner-wetted",
                        PowerSurfaceRole::DivertorWettedTarget,
                        0.3,
                    ),
                    share_of_divertor_power: 0.5,
                    peak_to_average_factor: 2.0,
                    assumptions: vec!["Synthetic target model".to_string()],
                },
            ],
        }
    }

    #[test]
    fn conserves_declared_power_and_uses_distinct_surface_areas() {
        let result = map_qecon_to_divertor_loads(&request()).unwrap();
        assert!((result.total_fusion_power_mw - 37.0).abs() < 1.0e-12);
        assert!((result.alpha_heating_power_mw - 7.4).abs() < 1.0e-12);
        assert!((result.separatrix_crossing_power_mw - 9.0).abs() < 1.0e-12);
        assert!((result.sol_radiated_power_mw - 4.5).abs() < 1.0e-12);
        assert!((result.divertor_routed_power_mw - 3.6).abs() < 1.0e-12);
        assert!((result.other_post_sol_power_mw - 0.9).abs() < 1.0e-12);
        assert!((result.divertor_targets[0].average_heat_flux_mw_per_m2 - 9.0).abs() < 1.0e-12);
        assert!((result.divertor_targets[0].estimated_peak_heat_flux_mw_per_m2 - 13.5).abs() < 1.0e-12);
        assert!((result.divertor_targets[1].average_heat_flux_mw_per_m2 - 6.0).abs() < 1.0e-12);
        assert!((result.divertor_targets[1].estimated_peak_heat_flux_mw_per_m2 - 12.0).abs() < 1.0e-12);
        assert!(result.interpretation_limits.len() >= 5);
    }

    #[test]
    fn stored_energy_accumulation_reduces_separatrix_power_and_release_increases_it() {
        let mut input = request();
        input.power_balance.stored_energy_change_rate_mw = 1.0;
        let accumulation = map_qecon_to_divertor_loads(&input).unwrap();
        input.power_balance.stored_energy_change_rate_mw = -1.0;
        let release = map_qecon_to_divertor_loads(&input).unwrap();
        assert!((accumulation.separatrix_crossing_power_mw - 8.0).abs() < 1.0e-12);
        assert!((release.separatrix_crossing_power_mw - 10.0).abs() < 1.0e-12);
    }

    #[test]
    fn rejects_surface_identity_or_role_mismatch() {
        let mut input = request();
        input.qecon_control_surface_id = "another-surface".to_string();
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input = request();
        input.divertor_targets[0].wetted_surface.role = PowerSurfaceRole::EnergyExtraction;
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input = request();
        input.divertor_targets[0].wetted_surface.surface_id = "energy-capture-S".to_string();
        assert!(map_qecon_to_divertor_loads(&input).is_err());
    }

    #[test]
    fn rejects_unbalanced_target_shares_and_nonphysical_peak_factor() {
        let mut input = request();
        input.divertor_targets[0].share_of_divertor_power = 0.4;
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input = request();
        input.divertor_targets[0].peak_to_average_factor = 0.99;
        assert!(map_qecon_to_divertor_loads(&input).is_err());
    }

    #[test]
    fn rejects_negative_separatrix_power() {
        let mut input = request();
        input.power_balance.core_radiated_power_mw = 100.0;
        assert!(map_qecon_to_divertor_loads(&input).is_err());
    }

    #[test]
    fn requires_explicit_time_window_for_window_average() {
        let mut input = request();
        input.time_basis.kind = PowerTimeBasisKind::WindowAverage;
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input.time_basis.averaging_window_seconds = Some(0.5);
        assert!(map_qecon_to_divertor_loads(&input).is_ok());
    }

    #[test]
    fn preserves_independent_evidence_classes_and_revisions() {
        let mut input = request();
        input.power_balance.evidence_class = QeconEvidenceClass::EngineeringEstimate;
        input.power_balance.source_locator = Some("https://example.org/edge-model".to_string());
        input.power_balance.model_revision = "edge-model-2026.1".to_string();
        input.divertor_targets[0].wetted_surface.evidence_class = QeconEvidenceClass::LiteratureReported;
        input.divertor_targets[0].wetted_surface.source_locator =
            Some("https://example.org/target-geometry-paper".to_string());
        input.divertor_targets[0].wetted_surface.model_revision = "target-geometry-v2".to_string();

        let result = map_qecon_to_divertor_loads(&input).unwrap();
        assert_eq!(
            result.power_balance_evidence_class,
            QeconEvidenceClass::EngineeringEstimate
        );
        assert_eq!(
            result.divertor_targets[0].evidence_class,
            QeconEvidenceClass::LiteratureReported
        );
        assert_eq!(
            result.divertor_targets[0].model_revision,
            "target-geometry-v2"
        );
        assert_eq!(
            result.qecon_evidence_class,
            QeconEvidenceClass::ScenarioAssumption
        );
    }

    #[test]
    fn rejects_malformed_evidence_receipts_and_incomplete_target_coverage() {
        let mut input = request();
        input.extraction_surface.input_digest_sha256 = "not-a-digest".to_string();
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input = request();
        input.divertor_targets.clear();
        assert!(map_qecon_to_divertor_loads(&input).is_err());

        input = request();
        input.power_balance.evidence_class = QeconEvidenceClass::Measured;
        input.power_balance.source_locator = None;
        assert!(map_qecon_to_divertor_loads(&input).is_err());
    }
}
