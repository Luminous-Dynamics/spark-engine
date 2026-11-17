//! A transparent, surface-normalized fusion-plant economic-gain model.
//!
//! This module implements the steady-state Q_econ framework described by
//! Whyte et al. (2026), "Criteria for the economic viability of fusion power
//! plants", Journal of Fusion Energy 45, article 49,
//! <https://doi.org/10.1007/s10894-026-00577-9>.
//!
//! It is deliberately separate from the plant-level LCOE estimator in
//! `plant_energy_balance`. Q_econ is a dimensionless ratio of modeled economic
//! gain to three modeled cost rates. It is a screening/design-window metric,
//! not a project-finance valuation and not evidence that a reactor can achieve
//! the specified power density, energy conversion, component fluence or uptime.
//!
//! All cost-rate outputs are in MUSD / (m² · calendar year). Input parameters
//! are normalized to the energy-extraction control surface S. The model assumes
//! temporal equilibrium, an energy-fluence-limited surface, and a repeating
//! operating/replacement cycle. It does not automatically add omitted costs,
//! taxes, depreciation, licensing, decommissioning, grid integration, market
//! dispatch constraints, or stochastic component failures.
//!
//! In particular, `net_energy_conversion_efficiency` must be a whole-system
//! net efficiency P_net / P_f, not merely a blanket thermal efficiency. The
//! caller must account for recirculating loads and all internal power
//! conversion before supplying it.

use crate::plant_energy_balance::PlantPowerBalance;
use serde::{Deserialize, Serialize};

/// Revision label for this implementation of the published steady-state model.
pub const QECON_MODEL_REVISION: &str = "qecon-whyte-2026-v0.1";

const GAIN_RATE_CONVERSION: f64 = 8.76e-3;
const TARGET_RATE_CONVERSION: f64 = 31.5;

/// Nine control parameters other than fusion power density.
///
/// Units follow the 2026 Q_econ paper. The real interest rate is expressed in
/// percentage points (e.g. 2.0 means 2%), while efficiency is a fraction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconDesignParameters {
    /// Energy-fluence limit of the energy-extraction surface S, MW·year/m².
    ///
    /// This is a design-specific damage/lifetime input, not a universal
    /// material constant. It should ultimately be constrained by neutronics,
    /// displacement damage, transmutation, thermal fatigue and other failure
    /// mechanisms for the actual component.
    pub surface_energy_fluence_limit_mw_year_per_m2: f64,
    /// Duration of the surface replacement interval, in calendar years.
    pub surface_replacement_time_years: f64,
    /// Whole-system net energy conversion efficiency P_net / P_f, in [0, 1].
    pub net_energy_conversion_efficiency: f64,
    /// Net selling value of the energy product after variable O&M, USD/MWh.
    ///
    /// It can represent electricity, heat, fuels or another energy-linked
    /// product, but the product mix and price basis must be documented.
    pub net_price_of_energy_usd_per_mwh: f64,
    /// Consumable target cost divided by target energy yield, USD/MJ.
    ///
    /// Use zero only when the concept genuinely has no separately modelled
    /// consumable target cost. Fuel-cycle costs outside the paper's target
    /// definition remain the responsibility of the scenario owner.
    pub target_cost_per_energy_yield_usd_per_mj: f64,
    /// Cost to remove, qualify, install and dispose of the replaceable surface
    /// and associated components, MUSD/m².
    pub surface_replacement_cost_musd_per_m2: f64,
    /// Plant construction/delivery plus fixed O&M cost normalized to S,
    /// MUSD/m². This is the paper's combined FPP areal-cost parameter.
    pub plant_areal_cost_musd_per_m2: f64,
    /// Averaged real interest rate in percentage points (2.0 means 2%).
    ///
    /// Values above -100% are mathematically admissible; the financial
    /// interpretation must still be justified by the scenario owner.
    pub real_interest_rate_percent: f64,
    /// Economic plant life used by the amortization term, in years.
    pub plant_lifetime_years: u32,
}

/// Inputs for one Q_econ scenario.
///
/// The ten controlling parameters are this power-density value plus the nine
/// fields in `design`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconInput {
    /// Time-averaged fusion power crossing the control surface per area, MW/m².
    pub fusion_power_density_mw_per_m2: f64,
    pub design: QeconDesignParameters,
}

/// Economic gain/cost rates and utilization for one declared scenario.
///
/// These outputs must be interpreted as a coherent set: a high Q_econ does not
/// imply high availability, high net electric output, a feasible materials
/// lifetime, or financeability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconResult {
    /// Time during which the surface is producing power divided by total
    /// operating-plus-replacement cycle time, in [0, 1].
    pub utilization_fraction: f64,
    /// Operating interval implied by fluence limit / fusion power density, y.
    pub surface_operating_interval_years: f64,
    /// Economic value rate from sale of the net energy product, MUSD/(m²·y).
    pub gain_rate_musd_per_m2_year: f64,
    /// Consumable target cost rate, MUSD/(m²·y).
    pub target_cost_rate_musd_per_m2_year: f64,
    /// Surface/component replacement cost rate, MUSD/(m²·y).
    pub surface_replacement_cost_rate_musd_per_m2_year: f64,
    /// Amortized construction, financing and fixed O&M rate, MUSD/(m²·y).
    pub fixed_cost_rate_musd_per_m2_year: f64,
    /// Net economic rate: gain less all three modelled cost rates,
    /// MUSD/(m²·y).
    pub net_economic_rate_musd_per_m2_year: f64,
    /// Q_econ = gain rate / (target + surface replacement + fixed cost).
    pub q_econ: f64,
    /// Paper's effective LCOE proxy, USD/MWh.
    ///
    /// This remains `None` when the net energy conversion efficiency or
    /// economic gain is zero. It excludes some costs included in a full NPV
    /// valuation and must not be presented as a bankable LCOE.
    pub effective_lcoe_usd_per_mwh: Option<f64>,
}

/// Evidence class attached to a scenario wrapper.
///
/// This label describes the source class for the inputs; it does not certify
/// the source or establish that a model prediction is physically correct.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QeconEvidenceClass {
    /// Values measured on an operating plant or relevant component.
    Measured,
    /// Values transcribed from a cited paper or published design study.
    LiteratureReported,
    /// Values supplied by a named engineering design estimate.
    EngineeringEstimate,
    /// Explicit hypothetical design assumptions.
    ScenarioAssumption,
    /// Synthetic data used only for software tests or method development.
    SyntheticFixture,
}

/// Evidence-carrying wrapper for a Q_econ model run.
///
/// `input_digest_sha256` must be the SHA-256 digest of a canonical
/// serialization of `input`. This module validates its syntax, but digest
/// creation and verification belong to the caller/evidence pipeline so that
/// one canonical serialization implementation is used across repositories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconScenario {
    pub scenario_id: String,
    pub model_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
    pub input: QeconInput,
}

/// Output of an evidence-carrying Q_econ run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QeconScenarioResult {
    pub scenario_id: String,
    pub model_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
    pub result: QeconResult,
}

/// Calculate the steady-state economic gain, cost rates and utilization.
///
/// Equations follow Whyte et al. (2026), Eqs. 23–29:
///
/// - U = 1 / (1 + tau_rep * (P_f/S) / X_S)
/// - C_gain = 8.76e-3 * POE_net * eta_E * (P_f/S) * U
/// - C_target = 31.5 * (c/Y)_target * (P_f/S) * U
/// - C_S,rep = (MUSD/S)_S * (P_f/S) / X_S * U
/// - C_fixed = (MUSD/S)_FPP * CRF(i, tau_life)
/// - Q_econ = C_gain / (C_target + C_S,rep + C_fixed)
///
/// The model uses calendar-year cost rates and includes the replacement
/// downtime through U. Do not multiply these cost rates by U a second time.
pub fn calculate_qecon(input: &QeconInput) -> Result<QeconResult, String> {
    validate_input(input)?;

    let power_density = input.fusion_power_density_mw_per_m2;
    let design = &input.design;
    let surface_operating_interval_years =
        design.surface_energy_fluence_limit_mw_year_per_m2 / power_density;
    let cycle_time_years =
        surface_operating_interval_years + design.surface_replacement_time_years;
    let utilization_fraction = surface_operating_interval_years / cycle_time_years;

    let gain_rate_musd_per_m2_year = GAIN_RATE_CONVERSION
        * design.net_price_of_energy_usd_per_mwh
        * design.net_energy_conversion_efficiency
        * power_density
        * utilization_fraction;

    let target_cost_rate_musd_per_m2_year = TARGET_RATE_CONVERSION
        * design.target_cost_per_energy_yield_usd_per_mj
        * power_density
        * utilization_fraction;

    let surface_replacement_cost_rate_musd_per_m2_year =
        design.surface_replacement_cost_musd_per_m2
            * power_density
            / design.surface_energy_fluence_limit_mw_year_per_m2
            * utilization_fraction;

    let capital_recovery_factor = capital_recovery_factor(
        design.real_interest_rate_percent,
        design.plant_lifetime_years,
    )?;
    let fixed_cost_rate_musd_per_m2_year =
        design.plant_areal_cost_musd_per_m2 * capital_recovery_factor;

    let total_cost_rate = target_cost_rate_musd_per_m2_year
        + surface_replacement_cost_rate_musd_per_m2_year
        + fixed_cost_rate_musd_per_m2_year;
    if !total_cost_rate.is_finite() || total_cost_rate <= 0.0 {
        return Err("total modelled economic cost rate must be finite and positive".to_string());
    }

    let net_economic_rate_musd_per_m2_year =
        gain_rate_musd_per_m2_year - total_cost_rate;
    let q_econ = gain_rate_musd_per_m2_year / total_cost_rate;
    let effective_lcoe_usd_per_mwh = if q_econ > 0.0 {
        Some(design.net_price_of_energy_usd_per_mwh / q_econ)
    } else {
        None
    };

    let outputs = [
        utilization_fraction,
        surface_operating_interval_years,
        gain_rate_musd_per_m2_year,
        target_cost_rate_musd_per_m2_year,
        surface_replacement_cost_rate_musd_per_m2_year,
        fixed_cost_rate_musd_per_m2_year,
        net_economic_rate_musd_per_m2_year,
        q_econ,
    ];
    if outputs.iter().any(|value| !value.is_finite())
        || effective_lcoe_usd_per_mwh.is_some_and(|value| !value.is_finite())
    {
        return Err("calculated Q_econ result is not finite".to_string());
    }

    Ok(QeconResult {
        utilization_fraction,
        surface_operating_interval_years,
        gain_rate_musd_per_m2_year,
        target_cost_rate_musd_per_m2_year,
        surface_replacement_cost_rate_musd_per_m2_year,
        fixed_cost_rate_musd_per_m2_year,
        net_economic_rate_musd_per_m2_year,
        q_econ,
        effective_lcoe_usd_per_mwh,
    })
}

/// Derive the Q_econ net-efficiency input from a complete electrical
/// plant-power ledger.
///
/// This is the electrical-product path only. It requires a complete auxiliary
/// boundary and calculates P_net / P_f using net grid export and fusion thermal
/// power. It deliberately refuses incomplete ledgers and values outside [0, 1].
pub fn net_electric_efficiency_from_power_balance(
    balance: &PlantPowerBalance,
) -> Result<f64, String> {
    if !balance.plant_boundary_complete {
        return Err("plant boundary is incomplete; net efficiency is unavailable".to_string());
    }
    let net_export_w = balance
        .net_electric_export_w
        .ok_or_else(|| "complete plant balance has no net electric export".to_string())?;
    if !balance.fusion_thermal_power_w.is_finite()
        || balance.fusion_thermal_power_w <= 0.0
        || !net_export_w.is_finite()
        || net_export_w < 0.0
    {
        return Err("fusion power must be positive and net export must be non-negative".to_string());
    }
    let efficiency = net_export_w / balance.fusion_thermal_power_w;
    if !efficiency.is_finite() || !(0.0..=1.0).contains(&efficiency) {
        return Err("derived net electric efficiency must be finite and in [0, 1]".to_string());
    }
    Ok(efficiency)
}

/// Solve the paper's analytic threshold for the required fusion power density.
///
/// This returns the unique positive P_f/S required for the requested Q_econ,
/// when a finite positive solution exists. It is a design-space threshold,
/// not evidence that the plasma, materials or cooling system can achieve it.
///
/// The current power-density field is intentionally not an argument: this
/// function uses only the nine non-density design parameters.
pub fn required_fusion_power_density_for_qecon(
    design: &QeconDesignParameters,
    target_q_econ: f64,
) -> Result<f64, String> {
    validate_design(design)?;
    if !target_q_econ.is_finite() || target_q_econ <= 0.0 {
        return Err("target_q_econ must be finite and positive".to_string());
    }

    let capital_recovery_factor =
        capital_recovery_factor(design.real_interest_rate_percent, design.plant_lifetime_years)?;
    let fixed_cost = design.plant_areal_cost_musd_per_m2 * capital_recovery_factor;

    let gain_coefficient = GAIN_RATE_CONVERSION
        * design.net_price_of_energy_usd_per_mwh
        * design.net_energy_conversion_efficiency;
    let target_cost_coefficient =
        TARGET_RATE_CONVERSION * design.target_cost_per_energy_yield_usd_per_mj;
    let replacement_cost_coefficient = design.surface_replacement_cost_musd_per_m2
        / design.surface_energy_fluence_limit_mw_year_per_m2;
    let replacement_time_coefficient = design.surface_replacement_time_years
        / design.surface_energy_fluence_limit_mw_year_per_m2;

    let denominator = gain_coefficient
        - target_q_econ
            * (target_cost_coefficient
                + replacement_cost_coefficient
                + replacement_time_coefficient * fixed_cost);
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(
            "no finite positive fusion power density reaches the requested Q_econ".to_string(),
        );
    }

    let required_density = target_q_econ * fixed_cost / denominator;
    if !required_density.is_finite() || required_density <= 0.0 {
        return Err("required fusion power density is not finite and positive".to_string());
    }
    Ok(required_density)
}

/// Calculate a scenario while preserving its evidence metadata.
pub fn calculate_qecon_scenario(
    scenario: &QeconScenario,
) -> Result<QeconScenarioResult, String> {
    if scenario.scenario_id.trim().is_empty() {
        return Err("scenario_id must not be empty".to_string());
    }
    if scenario.model_revision.trim().is_empty() {
        return Err("model_revision must not be empty".to_string());
    }
    if scenario.input_digest_sha256.len() != 64
        || !scenario
            .input_digest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("input_digest_sha256 must contain exactly 64 hexadecimal characters".to_string());
    }
    if scenario.assumptions.is_empty()
        || scenario.assumptions.iter().any(|item| item.trim().is_empty())
    {
        return Err("scenario must preserve at least one non-empty assumption".to_string());
    }
    if matches!(
        scenario.evidence_class,
        QeconEvidenceClass::Measured | QeconEvidenceClass::LiteratureReported
    ) && scenario
        .source_locator
        .as_deref()
        .is_none_or(|locator| locator.trim().is_empty())
    {
        return Err("measured and literature-reported scenarios require a source locator".to_string());
    }

    let result = calculate_qecon(&scenario.input)?;
    Ok(QeconScenarioResult {
        scenario_id: scenario.scenario_id.clone(),
        model_revision: scenario.model_revision.clone(),
        evidence_class: scenario.evidence_class,
        source_locator: scenario.source_locator.clone(),
        input_digest_sha256: scenario.input_digest_sha256.clone(),
        assumptions: scenario.assumptions.clone(),
        result,
    })
}

fn capital_recovery_factor(
    real_interest_rate_percent: f64,
    lifetime_years: u32,
) -> Result<f64, String> {
    let rate = real_interest_rate_percent / 100.0;
    if !rate.is_finite() || rate <= -1.0 {
        return Err("real_interest_rate_percent must be finite and greater than -100".to_string());
    }
    if lifetime_years == 0 {
        return Err("plant_lifetime_years must be positive".to_string());
    }

    let years = lifetime_years as f64;
    let factor = if rate == 0.0 {
        1.0 / years
    } else {
        rate / (1.0 - (1.0 + rate).powf(-years))
    };
    if !factor.is_finite() || factor <= 0.0 {
        return Err("capital recovery factor is not finite and positive".to_string());
    }
    Ok(factor)
}

fn validate_input(input: &QeconInput) -> Result<(), String> {
    if !input.fusion_power_density_mw_per_m2.is_finite()
        || input.fusion_power_density_mw_per_m2 <= 0.0
    {
        return Err("fusion_power_density_mw_per_m2 must be finite and positive".to_string());
    }
    validate_design(&input.design)
}

fn validate_design(design: &QeconDesignParameters) -> Result<(), String> {
    let non_negative = [
        (
            "surface_replacement_time_years",
            design.surface_replacement_time_years,
        ),
        (
            "net_price_of_energy_usd_per_mwh",
            design.net_price_of_energy_usd_per_mwh,
        ),
        (
            "target_cost_per_energy_yield_usd_per_mj",
            design.target_cost_per_energy_yield_usd_per_mj,
        ),
        (
            "surface_replacement_cost_musd_per_m2",
            design.surface_replacement_cost_musd_per_m2,
        ),
    ];
    for (name, value) in non_negative {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("{name} must be finite and non-negative"));
        }
    }

    if !design.surface_energy_fluence_limit_mw_year_per_m2.is_finite()
        || design.surface_energy_fluence_limit_mw_year_per_m2 <= 0.0
    {
        return Err(
            "surface_energy_fluence_limit_mw_year_per_m2 must be finite and positive".to_string(),
        );
    }
    if !design.net_energy_conversion_efficiency.is_finite()
        || !(0.0..=1.0).contains(&design.net_energy_conversion_efficiency)
    {
        return Err("net_energy_conversion_efficiency must be finite and in [0, 1]".to_string());
    }
    if !design.plant_areal_cost_musd_per_m2.is_finite()
        || design.plant_areal_cost_musd_per_m2 <= 0.0
    {
        return Err("plant_areal_cost_musd_per_m2 must be finite and positive".to_string());
    }
    if !design.real_interest_rate_percent.is_finite()
        || design.real_interest_rate_percent <= -100.0
    {
        return Err("real_interest_rate_percent must be finite and greater than -100".to_string());
    }
    if design.plant_lifetime_years == 0 {
        return Err("plant_lifetime_years must be positive".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literature_example_design() -> QeconDesignParameters {
        // Explicit reconstruction of the paper's illustrative assumptions:
        // X_S=3.125 MW-y/m², tau_rep=0.1 y, eta_E=0.4, POE_net=$100/MWh,
        // zero target cost, surface replacement cost=0.3 MUSD/m²,
        // plant areal cost=10 MUSD/m², real interest=2%, plant life=30 y.
        QeconDesignParameters {
            surface_energy_fluence_limit_mw_year_per_m2: 3.125,
            surface_replacement_time_years: 0.1,
            net_energy_conversion_efficiency: 0.4,
            net_price_of_energy_usd_per_mwh: 100.0,
            target_cost_per_energy_yield_usd_per_mj: 0.0,
            surface_replacement_cost_musd_per_m2: 0.3,
            plant_areal_cost_musd_per_m2: 10.0,
            real_interest_rate_percent: 2.0,
            plant_lifetime_years: 30,
        }
    }

    fn literature_example_input() -> QeconInput {
        QeconInput {
            fusion_power_density_mw_per_m2: 3.7,
            design: literature_example_design(),
        }
    }

    #[test]
    fn reproduces_published_qecon_design_window_example() {
        let result = calculate_qecon(&literature_example_input()).unwrap();

        // The paper reports Q_econ ~= 1.5 at P_f/S ~= 3.7 MW/m² for
        // POE_net=$100/MWh and an effective LCOE of approximately $67/MWh.
        assert!((result.q_econ - 1.5).abs() < 0.04);
        assert!((result.utilization_fraction - 0.894).abs() < 0.002);
        let effective_lcoe = result.effective_lcoe_usd_per_mwh.unwrap();
        assert!((effective_lcoe - 67.0).abs() < 2.0);
        assert!(result.net_economic_rate_musd_per_m2_year > 0.0);
    }

    #[test]
    fn solves_break_even_and_qecon_one_point_five_thresholds() {
        let design = literature_example_design();
        let break_even =
            required_fusion_power_density_for_qecon(&design, 1.0).unwrap();
        let q15 = required_fusion_power_density_for_qecon(&design, 1.5).unwrap();

        // The paper's illustrative solution reports approximately 1.9 MW/m²
        // at break-even and approximately 3.7 MW/m² for Q_econ ~= 1.5.
        assert!((break_even - 1.9).abs() < 0.15);
        assert!((q15 - 3.7).abs() < 0.2);
        assert!(q15 > break_even);
    }

    #[test]
    fn utilization_falls_as_power_density_increases() {
        let design = literature_example_design();
        let low = calculate_qecon(&QeconInput {
            fusion_power_density_mw_per_m2: 1.0,
            design: design.clone(),
        })
        .unwrap();
        let high = calculate_qecon(&QeconInput {
            fusion_power_density_mw_per_m2: 5.0,
            design,
        })
        .unwrap();

        assert!(high.utilization_fraction < low.utilization_fraction);
        assert!(high.q_econ > low.q_econ);
    }

    #[test]
    fn zero_replacement_interval_has_unit_surface_utilization() {
        let mut design = literature_example_design();
        design.surface_replacement_time_years = 0.0;
        let result = calculate_qecon(&QeconInput {
            fusion_power_density_mw_per_m2: 2.0,
            design,
        })
        .unwrap();
        assert!((result.utilization_fraction - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_real_interest_rate_uses_linear_amortization() {
        let mut design = literature_example_design();
        design.real_interest_rate_percent = 0.0;
        let result = calculate_qecon(&QeconInput {
            fusion_power_density_mw_per_m2: 2.0,
            design: design.clone(),
        })
        .unwrap();
        assert!((result.fixed_cost_rate_musd_per_m2_year
            - design.plant_areal_cost_musd_per_m2 / f64::from(design.plant_lifetime_years))
        .abs()
            < 1e-12);
    }

    #[test]
    fn rejects_inputs_outside_physical_or_model_domains() {
        let mut input = literature_example_input();
        input.design.net_energy_conversion_efficiency = 1.01;
        assert!(calculate_qecon(&input).is_err());

        let mut input = literature_example_input();
        input.design.surface_energy_fluence_limit_mw_year_per_m2 = f64::NAN;
        assert!(calculate_qecon(&input).is_err());

        let mut input = literature_example_input();
        input.fusion_power_density_mw_per_m2 = 0.0;
        assert!(calculate_qecon(&input).is_err());

        let mut input = literature_example_input();
        input.design.plant_lifetime_years = 0;
        assert!(calculate_qecon(&input).is_err());
    }

    #[test]
    fn rejects_target_qecon_without_finite_positive_solution() {
        let mut design = literature_example_design();
        design.net_energy_conversion_efficiency = 0.0;
        assert!(required_fusion_power_density_for_qecon(&design, 1.0).is_err());
        assert!(required_fusion_power_density_for_qecon(&design, f64::NAN).is_err());
    }

    #[test]
    fn derives_qecon_efficiency_only_from_complete_plant_power_balance() {
        use crate::plant_energy_balance::{
            PlantPowerBalanceInput, calculate_plant_power_balance,
        };

        let complete = PlantPowerBalanceInput {
            fusion_thermal_power_w: 1_000_000_000.0,
            primary_energy_to_medium_w: 50_000_000.0,
            primary_driver_wall_plug_power_w: 100_000_000.0,
            thermal_capture_fraction: 0.9,
            thermal_to_electric_efficiency: 0.4,
            direct_electric_output_w: 0.0,
            magnet_power_w: 20_000_000.0,
            cryogenic_power_w: 10_000_000.0,
            vacuum_and_pumping_power_w: 5_000_000.0,
            fuel_cycle_power_w: 10_000_000.0,
            cooling_and_heat_rejection_power_w: 15_000_000.0,
            other_auxiliary_power_w: 5_000_000.0,
            all_plant_auxiliaries_accounted_for: true,
        };
        let balance = calculate_plant_power_balance(&complete).unwrap();
        let efficiency = net_electric_efficiency_from_power_balance(&balance).unwrap();
        assert!((efficiency - 0.195).abs() < 1e-12);

        let mut incomplete = complete;
        incomplete.all_plant_auxiliaries_accounted_for = false;
        let partial = calculate_plant_power_balance(&incomplete).unwrap();
        assert!(net_electric_efficiency_from_power_balance(&partial).is_err());
    }

    #[test]
    fn evidence_wrapper_requires_digest_assumptions_and_sources() {
        let scenario = QeconScenario {
            scenario_id: "literature-example-2026".to_string(),
            model_revision: QECON_MODEL_REVISION.to_string(),
            evidence_class: QeconEvidenceClass::LiteratureReported,
            source_locator: Some("doi:10.1007/s10894-026-00577-9".to_string()),
            input_digest_sha256: "a".repeat(64),
            assumptions: vec!["Illustrative steady-state parameter set".to_string()],
            input: literature_example_input(),
        };
        let result = calculate_qecon_scenario(&scenario).unwrap();
        assert_eq!(result.scenario_id, scenario.scenario_id);
        assert_eq!(result.input_digest_sha256, scenario.input_digest_sha256);

        let mut invalid = scenario.clone();
        invalid.input_digest_sha256 = "not-a-digest".to_string();
        assert!(calculate_qecon_scenario(&invalid).is_err());

        let mut invalid = scenario.clone();
        invalid.source_locator = None;
        assert!(calculate_qecon_scenario(&invalid).is_err());

        let mut invalid = scenario;
        invalid.assumptions.clear();
        assert!(calculate_qecon_scenario(&invalid).is_err());
    }

    #[test]
    fn zero_energy_conversion_keeps_lcoe_undefined_not_infinite() {
        let mut design = literature_example_design();
        design.net_energy_conversion_efficiency = 0.0;
        let result = calculate_qecon(&QeconInput {
            fusion_power_density_mw_per_m2: 2.0,
            design,
        })
        .unwrap();
        assert_eq!(result.q_econ, 0.0);
        assert_eq!(result.effective_lcoe_usd_per_mwh, None);
    }
}
