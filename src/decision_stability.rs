//! Decision-stability benchmark controls for synthetic fusion architecture fixtures.
//!
//! These validators exercise decision semantics only. The checked-in fixtures are
//! invented analytical controls, not reactor predictions, measured data, or UQ calibration.
//! Production model outputs must continue to use the Spark case-package evidence contract.

use std::collections::{BTreeSet, HashMap};

use serde_json::Value;

pub const DECISION_STABILITY_VALIDATOR_REVISION: &str =
    "fusion-decision-stability-validator-v0.1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateDecision {
    pub candidate_id: String,
    pub model_set_disposition: String,
    pub robustly_feasible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionCliffSummary {
    pub candidates: Vec<CandidateDecision>,
    pub robust_feasible_set: Vec<String>,
    pub nominal_single_model_preference: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusSummary {
    pub model_count: usize,
    pub minimum_estimate: f64,
    pub maximum_estimate: f64,
    pub estimate_range: f64,
    pub dependence_cluster_count: usize,
    pub reference_error_minimum: f64,
    pub reference_error_maximum: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformalArithmeticSummary {
    pub rank_cases: usize,
    pub insufficient_calibration_cases: usize,
    pub coverage_composition_cases: usize,
}

fn parse_json(raw: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(raw).map_err(|error| format!("{label} is invalid JSON: {error}"))
}

fn required<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{context}.{key} is required"))
}

fn required_str(value: &Value, key: &str, context: &str) -> Result<String, String> {
    required(value, key, context)?
        .as_str()
        .filter(|item| !item.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("{context}.{key} must be a non-empty string"))
}

fn required_f64(value: &Value, key: &str, context: &str) -> Result<f64, String> {
    let number = required(value, key, context)?
        .as_f64()
        .ok_or_else(|| format!("{context}.{key} must be a number"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("{context}.{key} must be finite"))
    }
}

fn required_array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    required(value, key, context)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{context}.{key} must be an array"))
}

fn value_for_observable(evaluation: &Value, observable_id: &str) -> Result<f64, String> {
    let (key, context) = match observable_id {
        "net_export" => ("net_export_mwe", "evaluation"),
        "maintenance_clearance" => ("maintenance_clearance_m", "evaluation"),
        _ => return Err(format!("unsupported synthetic hard-gate observable {observable_id}")),
    };
    required_f64(evaluation, key, context)
}

fn evaluate_hard_gates(evaluation: &Value, constraints: &[Value]) -> Result<bool, String> {
    let mut passed = true;
    for constraint in constraints {
        let context = "hard_constraint";
        let observable = required_str(constraint, "observable_id", context)?;
        let operator = required_str(constraint, "operator", context)?;
        if operator != "greater_than_or_equal" {
            return Err(format!("unsupported synthetic hard-gate operator {operator}"));
        }
        let threshold = required_f64(constraint, "threshold", context)?;
        let actual = value_for_observable(evaluation, &observable)?;
        passed &= actual >= threshold;
    }
    Ok(passed)
}

/// Evaluate the synthetic model-choice design-cliff fixture.
///
/// The function refuses probability assignments to model variants. It computes the
/// finite observed model envelope only; it does not infer a probabilistic distribution.
pub fn validate_model_choice_cliff(raw: &str) -> Result<DecisionCliffSummary, String> {
    let root = parse_json(raw, "model-choice fixture")?;
    let fixture_class = required_str(&root, "fixture_class", "root")?;
    if fixture_class != "synthetic_analytic" {
        return Err("model-choice fixture must remain synthetic_analytic".to_string());
    }

    let profile = required(&root, "comparison_profile", "root")?;
    if !required(profile, "model_probabilities", "comparison_profile")?.is_null() {
        return Err("model variants must not receive invented probabilities".to_string());
    }
    if required_str(profile, "probabilistic_aggregation", "comparison_profile")? != "forbidden" {
        return Err("probabilistic aggregation must be explicitly forbidden for this model set".to_string());
    }

    let candidates = required_array(profile, "candidate_set", "comparison_profile")?;
    let models = required_array(profile, "model_set", "comparison_profile")?;
    let constraints = required_array(profile, "hard_constraints", "comparison_profile")?;
    if candidates.is_empty() || models.len() < 2 || constraints.is_empty() {
        return Err("candidate, model and hard-constraint sets must be non-empty".to_string());
    }

    let candidate_ids = candidates
        .iter()
        .map(|value| value.as_str().map(ToString::to_string)
            .ok_or_else(|| "candidate_set entries must be strings".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let model_ids = models
        .iter()
        .map(|value| value.as_str().map(ToString::to_string)
            .ok_or_else(|| "model_set entries must be strings".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;

    if candidate_ids.len() != candidates.len() || model_ids.len() != models.len() {
        return Err("candidate and model identifiers must be unique".to_string());
    }

    let evaluations = required_array(&root, "evaluations", "root")?;
    let mut by_candidate: HashMap<String, Vec<(String, bool, f64, f64)>> = HashMap::new();
    let mut seen_pairs = BTreeSet::new();

    for evaluation in evaluations {
        let candidate_id = required_str(evaluation, "candidate_id", "evaluation")?;
        let model_id = required_str(evaluation, "model_id", "evaluation")?;
        if !candidate_ids.contains(&candidate_id) || !model_ids.contains(&model_id) {
            return Err(format!("unknown candidate/model pair {candidate_id}/{model_id}"));
        }
        if !seen_pairs.insert((candidate_id.clone(), model_id.clone())) {
            return Err(format!("duplicate evaluation for {candidate_id}/{model_id}"));
        }
        let passed = evaluate_hard_gates(evaluation, constraints)?;
        let net_export = required_f64(evaluation, "net_export_mwe", "evaluation")?;
        let cost = required_f64(evaluation, "lifecycle_cost", "evaluation")?;
        by_candidate.entry(candidate_id).or_default()
            .push((model_id, passed, net_export, cost));
    }

    let expected = required(&root, "expected", "root")?;
    let expected_dispositions = required(expected, "candidate_dispositions", "expected")?;
    let mut summaries = Vec::new();
    let mut robust_feasible_set = Vec::new();

    for candidate_id in &candidate_ids {
        let rows = by_candidate.get(candidate_id)
            .ok_or_else(|| format!("candidate {candidate_id} has no model evaluations"))?;
        if rows.len() != model_ids.len() {
            return Err(format!("candidate {candidate_id} does not have one result for every registered model"));
        }
        let row_models: BTreeSet<_> = rows.iter().map(|row| row.0.clone()).collect();
        if row_models != model_ids {
            return Err(format!("candidate {candidate_id} model coverage does not match the registered model set"));
        }

        let pass_count = rows.iter().filter(|row| row.1).count();
        let robustly_feasible = pass_count == rows.len();
        let disposition = if robustly_feasible {
            "AllModelsPassObservedEnvelope"
        } else if pass_count > 0 {
            "DecisionSplit"
        } else {
            "AllModelsFailObservedEnvelope"
        };

        let expected_candidate = required(expected_dispositions, candidate_id, "expected.candidate_dispositions")?;
        let expected_disposition = required_str(expected_candidate, "model_set_disposition", "expected candidate")?;
        let expected_robust = required(expected_candidate, "robustly_feasible", "expected candidate")?
            .as_bool()
            .ok_or_else(|| "expected.robustly_feasible must be a boolean".to_string())?;
        if expected_disposition != disposition || expected_robust != robustly_feasible {
            return Err(format!("candidate {candidate_id} expected disposition does not match computed result"));
        }

        if robustly_feasible {
            robust_feasible_set.push(candidate_id.clone());
        }
        summaries.push(CandidateDecision {
            candidate_id: candidate_id.clone(),
            model_set_disposition: disposition.to_string(),
            robustly_feasible,
        });
    }

    let mut nominal_m1 = evaluations.iter()
        .filter(|evaluation| evaluation.get("model_id").and_then(Value::as_str) == Some("model-m1"))
        .map(|evaluation| {
            Ok((
                required_str(evaluation, "candidate_id", "evaluation")?,
                required_f64(evaluation, "net_export_mwe", "evaluation")?,
                required_f64(evaluation, "lifecycle_cost", "evaluation")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    nominal_m1.sort_by(|left, right| {
        right.1.total_cmp(&left.1).then_with(|| left.2.total_cmp(&right.2))
    });
    let nominal_preference = nominal_m1.first()
        .map(|row| row.0.clone())
        .ok_or_else(|| "model-m1 baseline evaluations are missing".to_string())?;
    let expected_preference = required_str(expected, "nominal_single_model_m1_preference", "expected")?;
    if nominal_preference != expected_preference {
        return Err("nominal single-model preference does not match the known answer".to_string());
    }

    robust_feasible_set.sort();
    let expected_robust = required_array(expected, "robust_feasible_set", "expected")?
        .iter()
        .map(|value| value.as_str().map(ToString::to_string)
            .ok_or_else(|| "robust_feasible_set entries must be strings".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if robust_feasible_set.iter().cloned().collect::<BTreeSet<_>>() != expected_robust {
        return Err("robust feasible set does not match the known answer".to_string());
    }

    summaries.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    Ok(DecisionCliffSummary {
        candidates: summaries,
        robust_feasible_set,
        nominal_single_model_preference: nominal_preference,
    })
}

/// Audit the synthetic consensus trap and count distinct registered source/calibration
/// dependence groups, not solver labels or execution runners.
pub fn validate_common_mode_consensus(raw: &str) -> Result<ConsensusSummary, String> {
    let root = parse_json(raw, "common-mode fixture")?;
    if required_str(&root, "fixture_class", "root")? != "synthetic_analytic" {
        return Err("common-mode fixture must remain synthetic_analytic".to_string());
    }
    let observable = required(&root, "observable_contract", "root")?;
    if required_str(observable, "unit", "observable_contract")? != "MWe" {
        return Err("common-mode fixture unit must be MWe".to_string());
    }

    let models = required_array(&root, "registered_models", "root")?;
    if models.len() < 2 {
        return Err("common-mode fixture requires at least two model labels".to_string());
    }
    let mut values = Vec::with_capacity(models.len());
    let mut dependence_groups = BTreeSet::new();
    let mut model_ids = BTreeSet::new();

    for model in models {
        let model_id = required_str(model, "model_id", "registered_model")?;
        if !model_ids.insert(model_id) {
            return Err("registered model IDs must be unique".to_string());
        }
        let source_family = required_str(model, "source_family", "registered_model")?;
        let calibration_group = required_str(model, "calibration_group", "registered_model")?;
        let _runner = required_str(model, "execution_lineage", "registered_model")?;
        dependence_groups.insert((source_family, calibration_group));
        values.push(required_f64(model, "estimate", "registered_model")?);
    }

    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let reference = required(&root, "held_reference", "root")?;
    if required_str(reference, "visibility", "held_reference")? != "evaluator_only" {
        return Err("held reference must remain evaluator_only".to_string());
    }
    let reference_value = required_f64(reference, "value", "held_reference")?;
    let expected = required(&root, "expected", "root")?;

    let expected_envelope = required(expected, "model_envelope", "expected")?;
    let expected_min = required_f64(expected_envelope, "minimum", "model_envelope")?;
    let expected_max = required_f64(expected_envelope, "maximum", "model_envelope")?;
    let expected_range = required_f64(expected_envelope, "range", "model_envelope")?;
    let range = maximum - minimum;
    if minimum != expected_min || maximum != expected_max || range != expected_range {
        return Err("common-mode model envelope does not match the known answer".to_string());
    }

    let expected_groups = required(expected, "shared_dependence_groups", "expected")?
        .as_array().ok_or_else(|| "shared_dependence_groups must be an array".to_string())?;
    let expected_group_set: BTreeSet<_> = expected_groups.iter()
        .map(|value| value.as_str().map(ToString::to_string)
            .ok_or_else(|| "shared_dependence_groups entries must be strings".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actual_group_labels: BTreeSet<_> = dependence_groups.iter()
        .flat_map(|(source, calibration)| [source.clone(), calibration.clone()])
        .collect();
    if actual_group_labels != expected_group_set {
        return Err("common-mode dependence labels do not match the known answer".to_string());
    }

    let expected_count = required(expected, "dependence_cluster_count", "expected")?
        .as_u64().ok_or_else(|| "dependence_cluster_count must be an integer".to_string())?;
    if dependence_groups.len() as u64 != expected_count {
        return Err("dependence cluster count must match distinct registered source/calibration groups".to_string());
    }
    let reference_error_minimum = reference_value - maximum;
    let reference_error_maximum = reference_value - minimum;
    let expected_error = required(expected, "reference_error_range", "expected")?;
    if reference_error_minimum != required_f64(expected_error, "minimum", "reference_error_range")?
        || reference_error_maximum != required_f64(expected_error, "maximum", "reference_error_range")? {
        return Err("common-mode reference error does not match the known answer".to_string());
    }

    Ok(ConsensusSummary {
        model_count: models.len(),
        minimum_estimate: minimum,
        maximum_estimate: maximum,
        estimate_range: range,
        dependence_cluster_count: dependence_groups.len(),
        reference_error_minimum,
        reference_error_maximum,
    })
}

/// Verify finite-sample split-conformal rank and conservative union-bound arithmetic.
/// These are arithmetic controls only; they do not validate exchangeability or coverage.
pub fn validate_conformal_coverage_arithmetic(
    raw: &str,
) -> Result<ConformalArithmeticSummary, String> {
    let root = parse_json(raw, "conformal arithmetic fixture")?;
    if required_str(&root, "fixture_class", "root")? != "synthetic_analytic" {
        return Err("conformal fixture must remain synthetic_analytic".to_string());
    }

    let rank_cases = required_array(&root, "conformal_rank_cases", "root")?;
    let mut insufficient = 0usize;
    for case in rank_cases {
        let n = required(case, "n", "rank_case")?.as_u64()
            .ok_or_else(|| "rank_case.n must be an unsigned integer".to_string())?;
        let coverage_bp = required(case, "coverage_basis_points", "rank_case")?.as_u64()
            .ok_or_else(|| "coverage_basis_points must be an unsigned integer".to_string())?;
        if coverage_bp == 0 || coverage_bp > 10_000 {
            return Err("coverage_basis_points must be in 1..=10000".to_string());
        }
        let numerator = n
            .checked_add(1)
            .and_then(|samples| samples.checked_mul(coverage_bp))
            .ok_or_else(|| "conformal rank arithmetic overflowed".to_string())?;
        let rank = numerator
            .checked_add(9_999)
            .ok_or_else(|| "conformal ceiling arithmetic overflowed".to_string())?
            / 10_000;
        let expected_rank = required(case, "rank_k", "rank_case")?.as_u64()
            .ok_or_else(|| "rank_k must be an unsigned integer".to_string())?;
        let available = rank <= n;
        let expected_available = required(case, "finite_bound_available", "rank_case")?
            .as_bool().ok_or_else(|| "finite_bound_available must be boolean".to_string())?;
        let expected_disposition = required_str(case, "expected_disposition", "rank_case")?;
        if rank != expected_rank || available != expected_available {
            return Err(format!("conformal rank case n={n}, coverage={coverage_bp} does not match known answer"));
        }
        if !available {
            insufficient += 1;
            if expected_disposition != "InsufficientCalibration" {
                return Err("unavailable conformal rank must be InsufficientCalibration".to_string());
            }
        } else if expected_disposition != "ArithmeticControlOnly" {
            return Err("finite synthetic conformal rank must remain ArithmeticControlOnly".to_string());
        }
    }

    let coverage_cases = required_array(&root, "coverage_composition_cases", "root")?;
    for case in coverage_cases {
        let parametric = required(case, "parametric_coverage_basis_points", "coverage_case")?
            .as_u64().ok_or_else(|| "parametric coverage must be an integer".to_string())?;
        let discrepancy = required(case, "discrepancy_coverage_basis_points", "coverage_case")?
            .as_u64().ok_or_else(|| "discrepancy coverage must be an integer".to_string())?;
        if parametric > 10_000 || discrepancy > 10_000 {
            return Err("coverage components must be in 0..=10000".to_string());
        }
        let uncovered = (10_000 - parametric) + (10_000 - discrepancy);
        let floor = 10_000_u64.saturating_sub(uncovered);
        let expected_floor = required(case, "union_bound_total_coverage_floor_basis_points", "coverage_case")?
            .as_u64().ok_or_else(|| "coverage floor must be an integer".to_string())?;
        if floor != expected_floor {
            return Err("union-bound coverage floor does not match the known answer".to_string());
        }
    }

    Ok(ConformalArithmeticSummary {
        rank_cases: rank_cases.len(),
        insufficient_calibration_cases: insufficient,
        coverage_composition_cases: coverage_cases.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        validate_common_mode_consensus, validate_conformal_coverage_arithmetic,
        validate_model_choice_cliff,
    };
    use serde_json::Value;

    const CLIFF: &str = include_str!(
        "../benchmarks/research/decision-stability/synthetic-model-choice-cliff-v0.1.json"
    );
    const CONSENSUS: &str = include_str!(
        "../benchmarks/research/decision-stability/synthetic-common-mode-consensus-v0.1.json"
    );
    const CONFORMAL: &str = include_str!(
        "../benchmarks/research/decision-stability/synthetic-conformal-coverage-arithmetic-v0.1.json"
    );

    #[test]
    fn model_choice_cliff_known_answer_passes() {
        let summary = validate_model_choice_cliff(CLIFF).expect("synthetic cliff is valid");
        assert_eq!(summary.robust_feasible_set, vec!["candidate-b".to_string()]);
        assert_eq!(summary.nominal_single_model_preference, "candidate-a");
        assert_eq!(summary.candidates.len(), 2);
    }

    #[test]
    fn model_choice_cliff_rejects_invented_probabilities() {
        let mut fixture: Value = serde_json::from_str(CLIFF).expect("fixture parses");
        fixture["comparison_profile"]["model_probabilities"] = serde_json::json!({
            "model-m1": 0.5,
            "model-m2": 0.5
        });
        assert!(validate_model_choice_cliff(&fixture.to_string()).is_err());
    }

    #[test]
    fn common_mode_consensus_counts_dependency_groups_not_model_names() {
        let summary = validate_common_mode_consensus(CONSENSUS).expect("synthetic consensus is valid");
        assert_eq!(summary.model_count, 3);
        assert_eq!(summary.dependence_cluster_count, 1);
        assert_eq!(summary.estimate_range, 2.0);
        assert_eq!(summary.reference_error_minimum, 28.0);
        assert_eq!(summary.reference_error_maximum, 30.0);
    }

    #[test]
    fn common_mode_consensus_rejects_false_independence_count() {
        let mut fixture: Value = serde_json::from_str(CONSENSUS).expect("fixture parses");
        fixture["expected"]["dependence_cluster_count"] = serde_json::json!(3);
        assert!(validate_common_mode_consensus(&fixture.to_string()).is_err());
    }

    #[test]
    fn conformal_ranks_and_union_bounds_match_known_answers() {
        let summary = validate_conformal_coverage_arithmetic(CONFORMAL)
            .expect("synthetic coverage arithmetic is valid");
        assert_eq!(summary.rank_cases, 3);
        assert_eq!(summary.insufficient_calibration_cases, 1);
        assert_eq!(summary.coverage_composition_cases, 2);
    }

    #[test]
    fn conformal_validator_rejects_clamped_finite_rank() {
        let mut fixture: Value = serde_json::from_str(CONFORMAL).expect("fixture parses");
        fixture["conformal_rank_cases"][0]["rank_k"] = serde_json::json!(18);
        fixture["conformal_rank_cases"][0]["finite_bound_available"] = serde_json::json!(true);
        fixture["conformal_rank_cases"][0]["expected_disposition"] = serde_json::json!("ArithmeticControlOnly");
        assert!(validate_conformal_coverage_arithmetic(&fixture.to_string()).is_err());
    }
}
