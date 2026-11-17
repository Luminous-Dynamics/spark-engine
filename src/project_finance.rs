//! Time-phased project cash flow for fusion plant scenarios.
//!
//! This module is deliberately separate from Q_econ and first-order plant
//! LCOE. Q_econ is a steady-state surface-normalized screening metric. This
//! module evaluates an explicitly supplied annual cash-flow path in one
//! declared currency and price basis.
//!
//! It is currently an unlevered project-cash-flow model. It does not build a
//! debt/equity waterfall, tax schedule, refinancing model, dispatch forecast or
//! construction stochastic process. Interest during construction can be
//! supplied as an explicit annual cost line, but the caller must ensure that
//! it is not double-counted through the discount-rate convention.
//!
//! Every period must include one explicit line for every cost category,
//! including categories whose amount is zero. This is intentionally stricter
//! than defaulting absent categories to zero.

use crate::qecon::QeconEvidenceClass;
use serde::{Deserialize, Serialize};

pub const PROJECT_FINANCE_MODEL_REVISION: &str = "fusion-project-cashflow-v0.1";

/// Whether all values and the discount rate are expressed in real or nominal
/// terms. This module does not perform inflation conversion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectCurrencyBasis {
    Real,
    Nominal,
}

/// Explicit annual project cost categories.
///
/// Every period must provide each category exactly once, even when the
/// amount is zero. This keeps omitted cost categories visible.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectCostCategory {
    ConstructionCapex,
    ComponentReplacement,
    FixedOperationsMaintenance,
    VariableOperationsMaintenance,
    FuelCycle,
    TargetsAndConsumables,
    RemoteMaintenance,
    HotCellAndCaskOperations,
    WasteDisposition,
    InsuranceAndLicensing,
    InterestDuringConstruction,
    Decommissioning,
    Taxes,
}

impl ProjectCostCategory {
    pub const ALL: [Self; 13] = [
        Self::ConstructionCapex,
        Self::ComponentReplacement,
        Self::FixedOperationsMaintenance,
        Self::VariableOperationsMaintenance,
        Self::FuelCycle,
        Self::TargetsAndConsumables,
        Self::RemoteMaintenance,
        Self::HotCellAndCaskOperations,
        Self::WasteDisposition,
        Self::InsuranceAndLicensing,
        Self::InterestDuringConstruction,
        Self::Decommissioning,
        Self::Taxes,
    ];
}

/// One explicit annual cost line in the declared currency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectCostLine {
    pub category: ProjectCostCategory,
    pub amount: f64,
}

/// Time-phased operating and financing inputs for one project year.
///
/// `year_index = 0` is the initial valuation year. Energy revenue is
/// calculated as production_mwh * realized_price_per_mwh; other-product and
/// salvage revenues are explicit cash amounts. Positive working-capital
/// investment consumes cash; release returns cash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectCashflowPeriod {
    pub year_index: u32,
    pub production_mwh: f64,
    pub realized_price_per_mwh: f64,
    pub other_product_revenue: f64,
    pub salvage_revenue: f64,
    pub working_capital_increase: f64,
    pub working_capital_release: f64,
    pub costs: Vec<ProjectCostLine>,
}

/// Evidence-carrying project cash-flow input.
///
/// All money amounts must use the same currency, currency year and real /
—nominal basis. The caller is responsible for converting them consistently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectFinanceInput {
    pub scenario_id: String,
    pub model_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    /// SHA-256 receipt for a canonical serialization of the full input.
    /// Syntax is validated here; cryptographic verification is external.
    pub input_digest_sha256: String,
    pub assumptions: Vec<String>,
    pub currency_code: String,
    pub currency_year: u32,
    pub currency_basis: ProjectCurrencyBasis,
    /// Annual discount rate in decimal form, e.g. 0.08 means 8%.
    /// Must use the same real/nominal basis as the cash-flow amounts.
    pub annual_discount_rate: f64,
    pub periods: Vec<ProjectCashflowPeriod>,
}

/// IRR availability status. IRR is deliberately withheld for non-standard
/// cash-flow patterns where it may be absent or non-unique.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectIrrStatus {
    UniqueRoot,
    NoSignChange,
    MultipleSignChanges,
    NoRootInSupportedInterval,
}

/// Calculated annual project cash-flow row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectCashflowResultPeriod {
    pub year_index: u32,
    pub energy_revenue: f64,
    pub other_product_revenue: f64,
    pub salvage_revenue: f64,
    pub total_revenue: f64,
    pub total_costs: f64,
    pub working_capital_increase: f64,
    pub working_capital_release: f64,
    pub working_capital_balance: f64,
    pub net_project_cashflow: f64,
    pub discounted_project_cashflow: f64,
    pub cumulative_project_cash_position_before_financing: f64,
}

/// Project cash-flow result, not a plant-level LCOE result or a levered equity
/// return model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectFinanceResult {
    pub scenario_id: String,
    pub model_revision: String,
    pub evidence_class: QeconEvidenceClass,
    pub source_locator: Option<String>,
    pub input_digest_sha256: String,
    pub currency_code: String,
    pub currency_year: u32,
    pub currency_basis: ProjectCurrencyBasis,
    pub annual_discount_rate: f64,
    pub net_present_value: f64,
    pub internal_rate_of_return: Option<f64>,
    pub irr_status: ProjectIrrStatus,
    pub peak_working_capital: f64,
    pub ending_working_capital: f64,
    /// Minimum cumulative unlevered project cash position before external
    /// debt/equity financing. Negative values indicate a funding trough.
    pub minimum_cumulative_cash_before_financing: f64,
    pub ending_cumulative_cash_before_financing: f64,
    pub periods: Vec<ProjectCashflowResultPeriod>,
    pub assumptions: Vec<String>,
}

/// Calculate unlevered project cash flow, NPV, a guarded IRR and liquidity
/// indicators from a fully declared annual schedule.
pub fn evaluate_project_cashflow(
    input: &ProjectFinanceInput,
) -> Result<ProjectFinanceResult, String> {
    validate_project_finance_input(input)?;

    let mut working_capital_balance = 0.0;
    let mut peak_working_capital: f64 = 0.0;
    let mut cumulative_cash = 0.0;
    let mut minimum_cumulative_cash = 0.0_f64;
    let mut net_cashflows = Vec::with_capacity(input.periods.len());
    let mut results = Vec::with_capacity(input.periods.len());
    let mut net_present_value = 0.0;

    for period in &input.periods {
        working_capital_balance +=
            period.working_capital_increase - period.working_capital_release;
        if working_capital_balance < -1e-8 {
            return Err(format!(
                "working-capital release exceeds accumulated balance in year {}",
                period.year_index
            ));
        }
        working_capital_balance = working_capital_balance.max(0.0);
        peak_working_capital = peak_working_capital.max(working_capital_balance);

        let energy_revenue = period.production_mwh * period.realized_price_per_mwh;
        let total_revenue =
            energy_revenue + period.other_product_revenue + period.salvage_revenue;
        let total_costs = period.costs.iter().map(|line| line.amount).sum::<f64>();
        let net_project_cashflow = total_revenue
            - total_costs
            - period.working_capital_increase
            + period.working_capital_release;
        let discount_factor = (1.0 + input.annual_discount_rate).powf(f64::from(period.year_index));
        if !discount_factor.is_finite() || discount_factor <= 0.0 {
            return Err(format!(
                "invalid discount factor in year {}",
                period.year_index
            ));
        }
        let discounted_project_cashflow = net_project_cashflow / discount_factor;
        net_present_value += discounted_project_cashflow;
        cumulative_cash += net_project_cashflow;
        minimum_cumulative_cash = minimum_cumulative_cash.min(cumulative_cash);
        net_cashflows.push(net_project_cashflow);
        results.push(ProjectCashflowResultPeriod {
            year_index: period.year_index,
            energy_revenue,
            other_product_revenue: period.other_product_revenue,
            salvage_revenue: period.salvage_revenue,
            total_revenue,
            total_costs,
            working_capital_increase: period.working_capital_increase,
            working_capital_release: period.working_capital_release,
            working_capital_balance,
            net_project_cashflow,
            discounted_project_cashflow,
            cumulative_project_cash_position_before_financing: cumulative_cash,
        });
    }

    if !net_present_value.is_finite()
        || !cumulative_cash.is_finite()
        || !peak_working_capital.is_finite()
    {
        return Err("project cash-flow outputs are not finite".to_string());
    }
    let (internal_rate_of_return, irr_status) = calculate_project_irr(&net_cashflows);

    Ok(ProjectFinanceResult {
        scenario_id: input.scenario_id.clone(),
        model_revision: input.model_revision.clone(),
        evidence_class: input.evidence_class,
        source_locator: input.source_locator.clone(),
        input_digest_sha256: input.input_digest_sha256.clone(),
        currency_code: input.currency_code.clone(),
        currency_year: input.currency_year,
        currency_basis: input.currency_basis,
        annual_discount_rate: input.annual_discount_rate,
        net_present_value,
        internal_rate_of_return,
        irr_status,
        peak_working_capital,
        ending_working_capital: working_capital_balance,
        minimum_cumulative_cash_before_financing: minimum_cumulative_cash,
        ending_cumulative_cash_before_financing: cumulative_cash,
        periods: results,
        assumptions: vec![
            "All annual amounts use the supplied currency code, currency year and real/nominal basis.".to_string(),
            "NPV is unlevered project NPV; no debt/equity waterfall or financing draw schedule is modelled.".to_string(),
            "Interest during construction is an explicit input cost; do not double-count it through the discount-rate convention.".to_string(),
            "The minimum cumulative project cash position is before external debt/equity funding and is not a solvency guarantee.".to_string(),
            "IRR is returned only for a single-sign-change cash-flow pattern with a root bracketed in [-0.99, 10.0].".to_string(),
            "Taxes are explicit cash-cost inputs; this version does not calculate tax rules, depreciation or tax-loss carryforwards.".to_string(),
            "Production and prices are caller-supplied annual assumptions; no market dispatch or price-formation model is included.".to_string(),
            "This model is distinct from steady-state Q_econ, plant LCOE and a bankable project-finance model.".to_string(),
        ],
    })
}

fn validate_project_finance_input(input: &ProjectFinanceInput) -> Result<(), String> {
    if input.scenario_id.trim().is_empty() {
        return Err("scenario_id must not be empty".to_string());
    }
    if input.model_revision != PROJECT_FINANCE_MODEL_REVISION {
        return Err(format!(
            "model_revision must match the active implementation ({PROJECT_FINANCE_MODEL_REVISION})"
        ));
    }
    if input.input_digest_sha256.len() != 64
        || !input
            .input_digest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("input_digest_sha256 must contain exactly 64 hexadecimal characters".to_string());
    }
    if input.assumptions.is_empty()
        || input.assumptions.iter().any(|item| item.trim().is_empty())
    {
        return Err("project cash-flow scenario requires non-empty assumptions".to_string());
    }
    if matches!(
        input.evidence_class,
        QeconEvidenceClass::Measured | QeconEvidenceClass::LiteratureReported
    ) && input
        .source_locator
        .as_ref()
        .is_none_or(|source| source.trim().is_empty())
    {
        return Err("measured and literature-reported cash-flow scenarios require a source locator".to_string());
    }
    if input.currency_code.len() != 3
        || !input.currency_code.bytes().all(|byte| byte.is_ascii_uppercase())
    {
        return Err("currency_code must be a three-letter uppercase code".to_string());
    }
    if input.currency_year < 1900 || input.currency_year > 2200 {
        return Err("currency_year must be in 1900..=2200".to_string());
    }
    if !input.annual_discount_rate.is_finite() || input.annual_discount_rate <= -1.0 {
        return Err("annual_discount_rate must be finite and greater than -100%".to_string());
    }
    if input.periods.is_empty() {
        return Err("at least one annual project cash-flow period is required".to_string());
    }

    for (expected_year, period) in input.periods.iter().enumerate() {
        if period.year_index != expected_year as u32 {
            return Err("cash-flow periods must be contiguous and start at year_index 0".to_string());
        }
        let values = [
            period.production_mwh,
            period.realized_price_per_mwh,
            period.other_product_revenue,
            period.salvage_revenue,
            period.working_capital_increase,
            period.working_capital_release,
        ];
        if values.iter().any(|value| !value.is_finite() || *value < 0.0) {
            return Err(format!(
                "production, revenue, salvage and working-capital values must be finite and non-negative in year {}",
                period.year_index
            ));
        }
        if period.working_capital_increase > 0.0 && period.working_capital_release > 0.0 {
            return Err(format!(
                "working-capital increase and release cannot both be positive in year {}",
                period.year_index
            ));
        }
        if period.costs.len() != ProjectCostCategory::ALL.len() {
            return Err(format!(
                "year {} must include all {} explicit cost categories",
                period.year_index,
                ProjectCostCategory::ALL.len()
            ));
        }
        let mut seen = [false; 13];
        for line in &period.costs {
            let index = cost_category_index(line.category);
            if seen[index] {
                return Err(format!(
                    "duplicate cost category {:?} in year {}",
                    line.category, period.year_index
                ));
            }
            seen[index] = true;
            if !line.amount.is_finite() || line.amount < 0.0 {
                return Err(format!(
                    "cost category {:?} must be finite and non-negative in year {}",
                    line.category, period.year_index
                ));
            }
        }
        if seen.iter().any(|present| !present) {
            return Err(format!(
                "year {} is missing an explicit cost category",
                period.year_index
            ));
        }
    }
    Ok(())
}

fn cost_category_index(category: ProjectCostCategory) -> usize {
    match category {
        ProjectCostCategory::ConstructionCapex => 0,
        ProjectCostCategory::ComponentReplacement => 1,
        ProjectCostCategory::FixedOperationsMaintenance => 2,
        ProjectCostCategory::VariableOperationsMaintenance => 3,
        ProjectCostCategory::FuelCycle => 4,
        ProjectCostCategory::TargetsAndConsumables => 5,
        ProjectCostCategory::RemoteMaintenance => 6,
        ProjectCostCategory::HotCellAndCaskOperations => 7,
        ProjectCostCategory::WasteDisposition => 8,
        ProjectCostCategory::InsuranceAndLicensing => 9,
        ProjectCostCategory::InterestDuringConstruction => 10,
        ProjectCostCategory::Decommissioning => 11,
        ProjectCostCategory::Taxes => 12,
    }
}

fn calculate_project_irr(cashflows: &[f64]) -> (Option<f64>, ProjectIrrStatus) {
    let nonzero_signs: Vec<bool> = cashflows
        .iter()
        .filter(|cashflow| **cashflow != 0.0)
        .map(|cashflow| *cashflow > 0.0)
        .collect();
    if nonzero_signs.len() < 2 {
        return (None, ProjectIrrStatus::NoSignChange);
    }
    let sign_changes = nonzero_signs
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count();
    if sign_changes == 0 {
        return (None, ProjectIrrStatus::NoSignChange);
    }
    if sign_changes > 1 {
        return (None, ProjectIrrStatus::MultipleSignChanges);
    }

    let low_rate = -0.99;
    let high_rate = 10.0;
    let low_npv = npv_at_rate(cashflows, low_rate);
    let high_npv = npv_at_rate(cashflows, high_rate);
    if !low_npv.is_finite()
        || !high_npv.is_finite()
        || low_npv == 0.0
        || high_npv == 0.0
        || low_npv.signum() == high_npv.signum()
    {
        return (None, ProjectIrrStatus::NoRootInSupportedInterval);
    }

    let mut left = low_rate;
    let mut right = high_rate;
    let mut left_value = low_npv;
    for _ in 0..160 {
        let midpoint = (left + right) * 0.5;
        let midpoint_value = npv_at_rate(cashflows, midpoint);
        if !midpoint_value.is_finite() {
            return (None, ProjectIrrStatus::NoRootInSupportedInterval);
        }
        if midpoint_value.abs() <= 1e-10 {
            return (Some(midpoint), ProjectIrrStatus::UniqueRoot);
        }
        if midpoint_value.signum() == left_value.signum() {
            left = midpoint;
            left_value = midpoint_value;
        } else {
            right = midpoint;
        }
    }
    (Some((left + right) * 0.5), ProjectIrrStatus::UniqueRoot)
}

fn npv_at_rate(cashflows: &[f64], rate: f64) -> f64 {
    cashflows
        .iter()
        .enumerate()
        .map(|(year, cashflow)| cashflow / (1.0 + rate).powf(year as f64))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_costs() -> Vec<ProjectCostLine> {
        ProjectCostCategory::ALL
            .into_iter()
            .map(|category| ProjectCostLine {
                category,
                amount: 0.0,
            })
            .collect()
    }

    fn input(periods: Vec<ProjectCashflowPeriod>) -> ProjectFinanceInput {
        ProjectFinanceInput {
            scenario_id: "cashflow-fixture-2026".to_string(),
            model_revision: PROJECT_FINANCE_MODEL_REVISION.to_string(),
            evidence_class: QeconEvidenceClass::ScenarioAssumption,
            source_locator: None,
            input_digest_sha256: "a".repeat(64),
            assumptions: vec![
                "Illustrative annual project cash-flow fixture".to_string(),
                "All monetary values are real 2025 USD".to_string(),
            ],
            currency_code: "USD".to_string(),
            currency_year: 2025,
            currency_basis: ProjectCurrencyBasis::Real,
            annual_discount_rate: 0.1,
            periods,
        }
    }

    fn period(year_index: u32) -> ProjectCashflowPeriod {
        ProjectCashflowPeriod {
            year_index,
            production_mwh: 0.0,
            realized_price_per_mwh: 0.0,
            other_product_revenue: 0.0,
            salvage_revenue: 0.0,
            working_capital_increase: 0.0,
            working_capital_release: 0.0,
            costs: zero_costs(),
        }
    }

    #[test]
    fn computes_npv_working_capital_and_liquidity_from_explicit_categories() {
        let mut year_zero = period(0);
        year_zero.costs.iter_mut()
            .find(|line| line.category == ProjectCostCategory::ConstructionCapex)
            .unwrap()
            .amount = 100.0;
        year_zero.working_capital_increase = 10.0;

        let mut year_one = period(1);
        year_one.production_mwh = 10.0;
        year_one.realized_price_per_mwh = 15.0;
        year_one.costs.iter_mut()
            .find(|line| line.category == ProjectCostCategory::VariableOperationsMaintenance)
            .unwrap()
            .amount = 20.0;
        year_one.working_capital_release = 10.0;

        let result = evaluate_project_cashflow(&input(vec![year_zero, year_one])).unwrap();
        assert!((result.net_present_value - 17.272_727_272_727_3).abs() < 1e-9);
        assert_eq!(result.peak_working_capital, 10.0);
        assert_eq!(result.ending_working_capital, 0.0);
        assert_eq!(result.minimum_cumulative_cash_before_financing, -110.0);
        assert_eq!(result.ending_cumulative_cash_before_financing, 30.0);
    }

    #[test]
    fn calculates_unique_irr_only_for_single_sign_change() {
        let mut year_zero = period(0);
        year_zero.costs.iter_mut()
            .find(|line| line.category == ProjectCostCategory::ConstructionCapex)
            .unwrap()
            .amount = 100.0;
        let mut year_one = period(1);
        year_one.production_mwh = 1.0;
        year_one.realized_price_per_mwh = 121.0;
        let result = evaluate_project_cashflow(&input(vec![year_zero, year_one])).unwrap();
        assert_eq!(result.irr_status, ProjectIrrStatus::UniqueRoot);
        assert!((result.internal_rate_of_return.unwrap() - 0.21).abs() < 1e-9);
    }

    #[test]
    fn withholds_irr_for_multiple_sign_changes() {
        let mut year_zero = period(0);
        year_zero.costs.iter_mut()
            .find(|line| line.category == ProjectCostCategory::ConstructionCapex)
            .unwrap()
            .amount = 100.0;
        let mut year_one = period(1);
        year_one.production_mwh = 1.0;
        year_one.realized_price_per_mwh = 230.0;
        let mut year_two = period(2);
        year_two.costs.iter_mut()
            .find(|line| line.category == ProjectCostCategory::Decommissioning)
            .unwrap()
            .amount = 132.0;
        let result = evaluate_project_cashflow(&input(vec![year_zero, year_one, year_two])).unwrap();
        assert_eq!(result.irr_status, ProjectIrrStatus::MultipleSignChanges);
        assert_eq!(result.internal_rate_of_return, None);
    }

    #[test]
    fn rejects_missing_or_duplicate_cost_categories() {
        let mut year = period(0);
        year.costs.pop();
        assert!(evaluate_project_cashflow(&input(vec![year])).is_err());

        let mut year = period(0);
        year.costs[1].category = year.costs[0].category;
        assert!(evaluate_project_cashflow(&input(vec![year])).is_err());
    }

    #[test]
    fn rejects_working_capital_release_that_exceeds_balance() {
        let mut year = period(0);
        year.working_capital_release = 1.0;
        assert!(evaluate_project_cashflow(&input(vec![year])).is_err());
    }

    #[test]
    fn rejects_invalid_currency_digest_and_negative_costs() {
        let mut scenario = input(vec![period(0)]);
        scenario.currency_code = "US".to_string();
        assert!(evaluate_project_cashflow(&scenario).is_err());

        scenario = input(vec![period(0)]);
        scenario.input_digest_sha256 = "invalid".to_string();
        assert!(evaluate_project_cashflow(&scenario).is_err());

        scenario = input(vec![period(0)]);
        scenario.periods[0].costs[0].amount = -1.0;
        assert!(evaluate_project_cashflow(&scenario).is_err());
    }
}
