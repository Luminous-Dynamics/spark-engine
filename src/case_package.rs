//! Semantic validation for fusion qualification case manifests.
//!
//! Checks cross-record semantics that JSON Schema cannot express portably:
//! identifier resolution, model-family isolation, unit/time/boundary contracts,
//! validation split leakage and conservation-ledger closure.
//!
//! A passing manifest is not experimentally validated or engineering-qualified.

use std::collections::{HashMap, HashSet};
use serde_json::Value;

pub const CASE_PACKAGE_SEMANTIC_VALIDATOR_REVISION: &str =
    "fusion-case-package-semantic-validator-v0.1";

const MODEL_FAMILIES: &[&str] = &[
    "synthetic_fixture", "lattice_confinement_fusion", "magnetic_confinement_fusion",
    "inertial_confinement_fusion", "magnetized_target_fusion", "fusion_plant_systems",
    "cross_family_contract",
];
const ADDITIVE_QUANTITIES: &[&str] = &[
    "power", "energy", "mass_flow", "mass", "particle_rate", "tritium_flow",
    "electric_power", "electrical_energy",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasePackageSummary {
    pub case_id: String,
    pub model_family: String,
    pub model_count: usize,
    pub boundary_count: usize,
    pub flow_count: usize,
    pub evidence_count: usize,
    pub accounting_check_count: usize,
}

pub fn validate_case_manifest_str(raw: &str) -> Result<CasePackageSummary, Vec<String>> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| vec![format!("manifest is not valid JSON: {error}")])?;
    validate_case_manifest(&value)
}

pub fn validate_case_manifest(value: &Value) -> Result<CasePackageSummary, Vec<String>> {
    let Some(root) = value.as_object() else {
        return Err(vec!["manifest root must be a JSON object".to_string()]);
    };
    let mut errors = Vec::new();
    let case_id = required_string(root.get("case_id"), "case_id", &mut errors);
    let model_family = required_string(root.get("model_family"), "model_family", &mut errors);
    if let Some(family) = model_family.as_deref() {
        if !MODEL_FAMILIES.contains(&family) {
            errors.push(format!("unsupported model_family {family}"));
        }
    }

    let time_bases = root.get("time_bases").and_then(Value::as_array);
    let boundaries = root.get("physical_boundaries").and_then(Value::as_array);
    let flows = root.get("flows").and_then(Value::as_array);
    let quantities = root.get("quantities").and_then(Value::as_array);
    let models = root.get("models").and_then(Value::as_array);
    let uncertainty = root.get("uncertainty_models").and_then(Value::as_array);
    let evidence = root.get("evidence_artifacts").and_then(Value::as_array);
    let accounting = root.get("accounting_checks").and_then(Value::as_array);
    let observables = root.get("observables").and_then(Value::as_array);

    let time_ids = collect_ids(time_bases, "time_bases", &mut errors);
    let boundary_ids = collect_ids(boundaries, "physical_boundaries", &mut errors);
    let flow_ids = collect_ids(flows, "flows", &mut errors);
    let model_ids = collect_ids(models, "models", &mut errors);
    let uncertainty_ids = collect_ids(uncertainty, "uncertainty_models", &mut errors);
    let evidence_ids = collect_ids(evidence, "evidence_artifacts", &mut errors);
    let _accounting_ids = collect_ids(accounting, "accounting_checks", &mut errors);
    let _observable_ids = collect_ids(observables, "observables", &mut errors);

    let evidence_by_id: HashMap<&str, &Value> = evidence.into_iter().flatten()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item))).collect();
    let time_by_id: HashMap<&str, &Value> = time_bases.into_iter().flatten()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item))).collect();
    let model_by_id: HashMap<&str, &Value> = models.into_iter().flatten()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item))).collect();
    let boundary_role_by_id: HashMap<&str, &str> = boundaries.into_iter().flatten()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item.get("role")?.as_str()?)))
        .collect();

    validate_time_bases(time_bases, &mut errors);
    validate_boundaries(boundaries, &evidence_ids, &mut errors);
    validate_models(models, model_family.as_deref(), &evidence_ids, &mut errors);
    validate_flows(flows, &boundary_ids, &boundary_role_by_id, &time_ids,
        &time_by_id, &evidence_ids, &uncertainty_ids, &mut errors);
    validate_quantities(quantities, &boundary_ids, &time_ids, &evidence_ids,
        &uncertainty_ids, &mut errors);
    validate_uncertainty(uncertainty, &evidence_ids, &mut errors);
    validate_splits(root.get("validation_plan"), &evidence_ids, &mut errors);
    validate_transfers(root.get("cross_family_transfers").and_then(Value::as_array),
        model_family.as_deref(), &model_ids, &model_by_id, &evidence_ids,
        &evidence_by_id, &mut errors);
    validate_accounting(accounting, flows, &flow_ids, &boundary_ids, &time_ids,
        &evidence_ids, &mut errors);
    validate_observables(observables, &evidence_ids, &mut errors);
    validate_qualification(root.get("qualification"), model_family.as_deref(),
        models, &evidence_ids, &evidence_by_id, &mut errors);

    if errors.is_empty() {
        Ok(CasePackageSummary {
            case_id: case_id.unwrap_or_default(),
            model_family: model_family.unwrap_or_default(),
            model_count: models.map_or(0, |items| items.len()),
            boundary_count: boundaries.map_or(0, |items| items.len()),
            flow_count: flows.map_or(0, |items| items.len()),
            evidence_count: evidence.map_or(0, |items| items.len()),
            accounting_check_count: accounting.map_or(0, |items| items.len()),
        })
    } else {
        Err(errors)
    }
}

fn required_string(value: Option<&Value>, name: &str, errors: &mut Vec<String>) -> Option<String> {
    match value.and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => Some(s.to_string()),
        _ => {
            errors.push(format!("{name} must be a non-empty string"));
            None
        }
    }
}

fn collect_ids(items: Option<&Vec<Value>>, label: &str, errors: &mut Vec<String>) -> HashSet<String> {
    let mut ids = HashSet::new();
    let Some(items) = items else {
        errors.push(format!("{label} must be an array"));
        return ids;
    };
    for (index, item) in items.iter().enumerate() {
        match item.get("id").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => {
                if !ids.insert(id.to_string()) {
                    errors.push(format!("duplicate identifier {id} in {label}"));
                }
            }
            _ => errors.push(format!("{label}[{index}].id must be a non-empty string")),
        }
    }
    ids
}

fn check_ref(id: Option<&str>, ids: &HashSet<String>, label: &str, context: &str,
    errors: &mut Vec<String>) {
    match id {
        Some(id) if ids.contains(id) => {}
        Some(id) => errors.push(format!("{context} references unknown {label} {id}")),
        None => errors.push(format!("{context} is missing a {label} identifier")),
    }
}

fn check_ref_array(item: &Value, field: &str, ids: &HashSet<String>, label: &str,
    context: &str, required: bool, nonempty: bool, errors: &mut Vec<String>) {
    let Some(value) = item.get(field) else {
        if required { errors.push(format!("{context}.{field} is required")); }
        return;
    };
    let Some(values) = value.as_array() else {
        errors.push(format!("{context}.{field} must be an array"));
        return;
    };
    if nonempty && values.is_empty() {
        errors.push(format!("{context}.{field} must not be empty"));
    }
    let mut seen = HashSet::new();
    for value in values {
        match value.as_str() {
            Some(id) => {
                if !seen.insert(id) {
                    errors.push(format!("{context}.{field} contains duplicate identifier {id}"));
                }
                check_ref(Some(id), ids, label, &format!("{context}.{field}"), errors);
            }
            None => errors.push(format!("{context}.{field} entries must be strings")),
        }
    }
}

fn validate_time_bases(items: Option<&Vec<Value>>, errors: &mut Vec<String>) {
    for (index, basis) in items.into_iter().flatten().enumerate() {
        let context = format!("time_bases[{index}]");
        let kind = basis.get("kind").and_then(Value::as_str);
        let duration = basis.get("duration_seconds").and_then(Value::as_f64);
        let sample = basis.get("sample_interval_seconds").and_then(Value::as_f64);
        match kind {
            Some("finite_window_average") if !duration.is_some_and(|v| v.is_finite() && v > 0.0) =>
                errors.push(format!("{context} requires a finite positive duration_seconds")),
            Some("time_series") => {
                if !duration.is_some_and(|v| v.is_finite() && v > 0.0) {
                    errors.push(format!("{context} time_series requires a positive duration_seconds"));
                }
                if !sample.is_some_and(|v| v.is_finite() && v > 0.0) {
                    errors.push(format!("{context} time_series requires a positive sample_interval_seconds"));
                }
            }
            Some("steady_state") if duration.is_some() =>
                errors.push(format!("{context} steady_state must not claim a finite duration")),
            None => errors.push(format!("{context}.kind must be a string")),
            _ => {}
        }
    }
}

fn validate_boundaries(items: Option<&Vec<Value>>, evidence_ids: &HashSet<String>,
    errors: &mut Vec<String>) {
    for (index, boundary) in items.into_iter().flatten().enumerate() {
        if let Some(geometry_id) = boundary.get("geometry_artifact_id").and_then(Value::as_str) {
            check_ref(Some(geometry_id), evidence_ids, "geometry evidence artifact",
                &format!("physical_boundaries[{index}]"), errors);
        }
    }
}

fn validate_models(items: Option<&Vec<Value>>, case_family: Option<&str>,
    evidence_ids: &HashSet<String>, errors: &mut Vec<String>) {
    for (index, model) in items.into_iter().flatten().enumerate() {
        let context = format!("models[{index}]");
        let family = model.get("model_family").and_then(Value::as_str);
        if !family.is_some_and(|f| MODEL_FAMILIES.contains(&f)) {
            errors.push(format!("{context}.model_family is missing or unsupported"));
        }
        if case_family != Some("cross_family_contract") && family.is_some() && family != case_family {
            errors.push(format!(
                "{context} family {family:?} differs from case family {case_family:?}; implicit cross-family transfer is forbidden"
            ));
        }
        if family == Some("cross_family_contract") {
            errors.push(format!("{context} must identify a concrete model family"));
        }
        check_ref_array(model, "input_artifact_ids", evidence_ids, "input evidence",
            &context, true, false, errors);
        check_ref_array(model, "output_artifact_ids", evidence_ids, "output evidence",
            &context, true, false, errors);
    }
}

fn validate_flows(items: Option<&Vec<Value>>, boundaries: &HashSet<String>,
    boundary_roles: &HashMap<&str, &str>, times: &HashSet<String>,
    time_by_id: &HashMap<&str, &Value>, evidence: &HashSet<String>,
    uncertainty: &HashSet<String>, errors: &mut Vec<String>) {
    for (index, flow) in items.into_iter().flatten().enumerate() {
        let context = format!("flows[{index}]");
        let from = flow.get("from_boundary_id").and_then(Value::as_str);
        let to = flow.get("to_boundary_id").and_then(Value::as_str);
        check_ref(from, boundaries, "boundary", &format!("{context}.from_boundary_id"), errors);
        check_ref(to, boundaries, "boundary", &format!("{context}.to_boundary_id"), errors);
        if from.is_some() && from == to {
            errors.push(format!("{context} cannot flow from a boundary to itself"));
        }
        let time_id = flow.get("time_basis_id").and_then(Value::as_str);
        check_ref(time_id, times, "time basis", &context, errors);
        check_ref_array(flow, "evidence_artifact_ids", evidence, "evidence artifact",
            &context, true, true, errors);
        if let Some(id) = flow.get("uncertainty_model_id").and_then(Value::as_str) {
            check_ref(Some(id), uncertainty, "uncertainty model", &context, errors);
        }

        let kind = flow.get("quantity_kind").and_then(Value::as_str);
        let surface = flow.get("surface_role").and_then(Value::as_str);
        let statistic = flow.get("spatial_statistic").and_then(Value::as_str);
        if matches!(kind, Some("heat_flux" | "fusion_power_density" | "divertor_heat_flux"))
            && (surface.is_none() || statistic.is_none()) {
            errors.push(format!("{context} surface-normalized quantity requires surface_role and spatial_statistic"));
        }
        if kind == Some("fusion_power_density") && surface != Some("energy_extraction_surface") {
            errors.push(format!("{context} fusion_power_density must use energy_extraction_surface"));
        }
        if kind == Some("divertor_heat_flux") && surface != Some("divertor_target_wetted_surface") {
            errors.push(format!("{context} divertor_heat_flux must use divertor_target_wetted_surface"));
        }
        if kind == Some("heat_flux") && !matches!(surface,
            Some("energy_extraction_surface" | "divertor_target_wetted_surface" | "first_wall" | "blanket" | "other")) {
            errors.push(format!("{context} heat_flux has missing or unsupported surface_role"));
        }
        if let Some(surface) = surface {
            let endpoint_matches = [from, to].into_iter().flatten()
                .any(|id| boundary_roles.get(id).copied() == Some(surface));
            if !endpoint_matches {
                errors.push(format!(
                    "{context} surface_role {surface} does not match either declared endpoint boundary"
                ));
            }
        }

        match flow.get("value").and_then(Value::as_f64) {
            Some(value) if value.is_finite() && value >= 0.0 => {}
            Some(_) => errors.push(format!("{context}.value must be finite and non-negative; reverse the flow direction instead of using a negative value")),
            None => errors.push(format!("{context}.value must be numeric")),
        }
        let unit_code = flow.get("unit").and_then(|u| u.get("code")).and_then(Value::as_str);
        if flow.get("unit").and_then(|u| u.get("system")).and_then(Value::as_str) != Some("UCUM")
            || unit_code.is_none_or(str::is_empty) {
            errors.push(format!("{context}.unit must declare a non-empty UCUM code"));
        } else if !unit_matches_quantity(kind, unit_code.unwrap_or_default()) {
            errors.push(format!(
                "{context} UCUM unit {} is incompatible with quantity_kind {}",
                unit_code.unwrap_or_default(),
                kind.unwrap_or("missing")
            ));
        }
        if let Some(time_id) = time_id {
            if time_by_id.get(time_id).and_then(|basis| basis.get("kind")).and_then(Value::as_str) == Some("time_series") {
                let series_id = flow.get("time_series_artifact_id").and_then(Value::as_str);
                check_ref(series_id, evidence, "time-series evidence artifact", &context, errors);
            } else if flow.get("time_series_artifact_id").is_some() {
                errors.push(format!("{context} provides a time-series artifact but its time basis is not time_series"));
            }
        }
    }
}


/// Conservative UCUM-dimension compatibility for common fusion engineering
/// quantities. This is deliberately a subset, not a general UCUM parser:
/// unsupported unit spellings fail closed instead of being silently coerced.
fn unit_matches_quantity(kind: Option<&str>, code: &str) -> bool {
    let power = ["W", "kW", "MW", "GW"].contains(&code);
    let energy = ["J", "kJ", "MJ", "GJ", "Wh", "kWh", "MWh", "GWh"].contains(&code);
    let surface_power_density = ["W.m-2", "kW.m-2", "MW.m-2"].contains(&code);
    let mass_flow = ["g.s-1", "kg.s-1"].contains(&code);
    let mass = ["g", "kg"].contains(&code);
    let particle_rate = code == "s-1";
    let temperature = ["K", "Cel"].contains(&code);
    let pressure = ["Pa", "kPa", "MPa", "bar"].contains(&code);
    match kind {
        Some("power" | "electric_power") => power,
        Some("energy" | "electrical_energy") => energy,
        Some("heat_flux" | "fusion_power_density" | "divertor_heat_flux") => surface_power_density,
        Some("mass_flow") => mass_flow,
        Some("tritium_flow") => mass_flow || particle_rate,
        Some("mass") => mass,
        Some("tritium_inventory") => mass || code == "1",
        Some("particle_rate") => particle_rate,
        Some("temperature") => temperature,
        Some("pressure") => pressure,
        Some("cost" | "price" | "probability" | "availability" | "stress" | "strain"
            | "damage" | "lifetime" | "other") => !code.trim().is_empty(),
        _ => false,
    }
}

fn validate_quantities(items: Option<&Vec<Value>>, boundaries: &HashSet<String>,
    times: &HashSet<String>, evidence: &HashSet<String>, uncertainty: &HashSet<String>,
    errors: &mut Vec<String>) {
    for (index, quantity) in items.into_iter().flatten().enumerate() {
        let context = format!("quantities[{index}]");
        match quantity.get("status").and_then(Value::as_str) {
            Some("known") => {
                if quantity.get("value").and_then(Value::as_f64).is_none_or(|v| !v.is_finite()) {
                    errors.push(format!("{context} known value must be finite and numeric"));
                }
                let unit_code = quantity.get("unit").and_then(|u| u.get("code")).and_then(Value::as_str);
                if quantity.get("unit").and_then(|u| u.get("system")).and_then(Value::as_str) != Some("UCUM")
                    || unit_code.is_none_or(str::is_empty) {
                    errors.push(format!("{context} known quantity requires a UCUM unit"));
                } else if !unit_matches_quantity(
                    quantity.get("quantity_kind").and_then(Value::as_str),
                    unit_code.unwrap_or_default()
                ) {
                    errors.push(format!(
                        "{context} UCUM unit {} is incompatible with quantity_kind {}",
                        unit_code.unwrap_or_default(),
                        quantity.get("quantity_kind").and_then(Value::as_str).unwrap_or("missing")
                    ));
                }
                check_ref(quantity.get("physical_boundary_id").and_then(Value::as_str), boundaries,
                    "physical boundary", &context, errors);
                check_ref(quantity.get("time_basis_id").and_then(Value::as_str), times,
                    "time basis", &context, errors);
                check_ref_array(quantity, "evidence_artifact_ids", evidence, "evidence artifact",
                    &context, true, true, errors);
                if quantity.get("unknown_reason").is_some() {
                    errors.push(format!("{context} known quantity must not carry unknown_reason"));
                }
            }
            Some("unknown" | "out_of_domain") => {
                if quantity.get("value").is_some() {
                    errors.push(format!("{context} unknown/out_of_domain quantity must not encode a numeric value"));
                }
                if quantity.get("unknown_reason").and_then(Value::as_str).is_none_or(str::is_empty) {
                    errors.push(format!("{context} unknown/out_of_domain quantity requires unknown_reason"));
                }
            }
            _ => errors.push(format!("{context}.status must be known, unknown or out_of_domain")),
        }
        if let Some(id) = quantity.get("uncertainty_model_id").and_then(Value::as_str) {
            check_ref(Some(id), uncertainty, "uncertainty model", &context, errors);
        }
    }
}

fn validate_uncertainty(items: Option<&Vec<Value>>, evidence: &HashSet<String>,
    errors: &mut Vec<String>) {
    for (index, item) in items.into_iter().flatten().enumerate() {
        check_ref_array(item, "source_artifact_ids", evidence, "uncertainty source evidence",
            &format!("uncertainty_models[{index}]"), false, false, errors);
    }
}

fn validate_splits(plan: Option<&Value>, evidence: &HashSet<String>, errors: &mut Vec<String>) {
    let Some(splits) = plan.and_then(|p| p.get("splits")).and_then(Value::as_array) else {
        errors.push("validation_plan.splits must be an array".to_string());
        return;
    };
    let mut split_ids = HashSet::new();
    let mut role_data: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (index, split) in splits.iter().enumerate() {
        let context = format!("validation_plan.splits[{index}]");
        if let Some(id) = split.get("id").and_then(Value::as_str) {
            if !split_ids.insert(id) {
                errors.push(format!("duplicate validation split id {id}"));
            }
        }
        let role = split.get("role").and_then(Value::as_str);
        let Some(data) = split.get("dataset_artifact_ids").and_then(Value::as_array) else {
            errors.push(format!("{context}.dataset_artifact_ids must be an array"));
            continue;
        };
        if data.is_empty() {
            errors.push(format!("{context}.dataset_artifact_ids must not be empty"));
        }
        for value in data {
            if let Some(id) = value.as_str() {
                check_ref(Some(id), evidence, "validation dataset evidence", &context, errors);
                if let Some(role) = role {
                    role_data.entry(role).or_default().insert(id);
                }
            } else {
                errors.push(format!("{context}.dataset_artifact_ids entries must be strings"));
            }
        }
    }
    for (left, right) in [("calibration", "validation"), ("calibration", "holdout"), ("validation", "holdout")] {
        if let (Some(a), Some(b)) = (role_data.get(left), role_data.get(right)) {
            for id in a.intersection(b) {
                errors.push(format!("validation data leakage: dataset {id} appears in both {left} and {right} splits"));
            }
        }
    }
}

fn validate_transfers(transfers: Option<&Vec<Value>>, case_family: Option<&str>,
    model_ids: &HashSet<String>, model_by_id: &HashMap<&str, &Value>,
    evidence_ids: &HashSet<String>, evidence_by_id: &HashMap<&str, &Value>,
    errors: &mut Vec<String>) {
    if case_family != Some("cross_family_contract") {
        if transfers.is_some_and(|items| !items.is_empty()) {
            errors.push("cross_family_transfers are only valid in cross_family_contract cases".to_string());
        }
        return;
    }
    let Some(transfers) = transfers.filter(|items| !items.is_empty()) else {
        errors.push("cross_family_contract requires at least one explicit reviewed transfer".to_string());
        return;
    };
    let mut transfer_ids = HashSet::new();
    for (index, transfer) in transfers.iter().enumerate() {
        let context = format!("cross_family_transfers[{index}]");
        let transfer_id = transfer.get("id").and_then(Value::as_str);
        if transfer_id.is_none_or(str::is_empty) {
            errors.push(format!("{context}.id must be a non-empty string"));
        } else if !transfer_ids.insert(transfer_id.unwrap_or_default()) {
            errors.push(format!("duplicate cross-family transfer id {}", transfer_id.unwrap_or_default()));
        }
        let source_id = transfer.get("source_model_id").and_then(Value::as_str);
        let destination_id = transfer.get("destination_model_id").and_then(Value::as_str);
        check_ref(source_id, model_ids, "source model", &context, errors);
        check_ref(destination_id, model_ids, "destination model", &context, errors);
        let source_family = transfer.get("source_family").and_then(Value::as_str);
        let destination_family = transfer.get("destination_family").and_then(Value::as_str);
        if source_family == destination_family || source_family == Some("cross_family_contract")
            || destination_family == Some("cross_family_contract")
            || !source_family.is_some_and(|family| MODEL_FAMILIES.contains(&family))
            || !destination_family.is_some_and(|family| MODEL_FAMILIES.contains(&family)) {
            errors.push(format!("{context} must connect two distinct supported concrete model families"));
        }
        if source_id.is_some() && source_id == destination_id {
            errors.push(format!("{context} cannot transfer from a model to itself"));
        }
        for field in ["rationale", "claim"] {
            if transfer.get(field).and_then(Value::as_str).is_none_or(str::is_empty) {
                errors.push(format!("{context}.{field} must be a non-empty string"));
            }
        }
        for (model_id, family, role) in [
            (source_id, source_family, "source"),
            (destination_id, destination_family, "destination"),
        ] {
            if let (Some(model_id), Some(family)) = (model_id, family) {
                if let Some(model) = model_by_id.get(model_id) {
                    if model.get("model_family").and_then(Value::as_str) != Some(family) {
                        errors.push(format!("{context} {role} family does not match model {model_id}"));
                    }
                }
            }
        }
        let review_id = transfer.get("independent_review_evidence_id").and_then(Value::as_str);
        check_ref(review_id, evidence_ids, "independent-review evidence", &context, errors);
        if let Some(review) = review_id.and_then(|id| evidence_by_id.get(id)) {
            let class = review.get("evidence_class").and_then(Value::as_str);
            if !matches!(class, Some("measured" | "code_to_code_comparison" | "experimental_validation" | "engineering_qualification")) {
                errors.push(format!("{context} independent-review evidence class is insufficient"));
            }
        }
    }
}

fn validate_accounting(checks: Option<&Vec<Value>>, flows: Option<&Vec<Value>>,
    flow_ids: &HashSet<String>, boundaries: &HashSet<String>, times: &HashSet<String>,
    evidence: &HashSet<String>, errors: &mut Vec<String>) {
    let flow_by_id: HashMap<&str, &Value> = flows.into_iter().flatten()
        .filter_map(|flow| Some((flow.get("id")?.as_str()?, flow))).collect();
    for (index, check) in checks.into_iter().flatten().enumerate() {
        let context = format!("accounting_checks[{index}]");
        let boundary = check.get("boundary_id").and_then(Value::as_str);
        let time = check.get("time_basis_id").and_then(Value::as_str);
        check_ref(boundary, boundaries, "accounting boundary", &context, errors);
        check_ref(time, times, "time basis", &context, errors);
        check_ref_array(check, "evidence_artifact_ids", evidence, "evidence artifact",
            &context, true, true, errors);

        let kind = check.get("quantity_kind").and_then(Value::as_str);
        if !kind.is_some_and(|k| ADDITIVE_QUANTITIES.contains(&k)) {
            errors.push(format!("{context} quantity_kind is not an additive conserved quantity"));
            continue;
        }
        let unit = check.get("unit").and_then(|u| u.get("code")).and_then(Value::as_str);
        if check.get("unit").and_then(|u| u.get("system")).and_then(Value::as_str) != Some("UCUM")
            || unit.is_none_or(str::is_empty) {
            errors.push(format!("{context}.unit must declare a UCUM code"));
        }
        let tolerance = check.get("relative_tolerance").and_then(Value::as_f64);
        if !tolerance.is_some_and(|v| v.is_finite() && v >= 0.0) {
            errors.push(format!("{context}.relative_tolerance must be finite and non-negative"));
        }

        let inbound = resolve_flows(check, "inbound_flow_ids", flow_ids, &flow_by_id, &context, errors);
        let outbound = resolve_flows(check, "outbound_flow_ids", flow_ids, &flow_by_id, &context, errors);
        let mut incoming_total = 0.0;
        let mut outgoing_total = 0.0;
        for (direction, resolved, total) in [
            ("inbound", inbound, &mut incoming_total),
            ("outbound", outbound, &mut outgoing_total),
        ] {
            for (id, flow) in resolved {
                let flow_context = format!("{context}.{direction}[{id}]");
                let expected_boundary = if direction == "inbound" {
                    flow.get("to_boundary_id").and_then(Value::as_str)
                } else {
                    flow.get("from_boundary_id").and_then(Value::as_str)
                };
                if expected_boundary != boundary {
                    errors.push(format!("{flow_context} has the wrong direction for accounting boundary"));
                }
                if flow.get("quantity_kind").and_then(Value::as_str) != kind {
                    errors.push(format!("{flow_context} quantity_kind differs from accounting check"));
                }
                if flow.get("unit").and_then(|u| u.get("code")).and_then(Value::as_str) != unit {
                    errors.push(format!("{flow_context} unit differs; implicit unit conversion is forbidden"));
                }
                if flow.get("time_basis_id").and_then(Value::as_str) != time {
                    errors.push(format!("{flow_context} time basis differs from accounting check"));
                }
                if let Some(value) = flow.get("value").and_then(Value::as_f64) {
                    *total += value;
                }
            }
        }
        if !incoming_total.is_finite() || !outgoing_total.is_finite() {
            errors.push(format!("{context} totals are not finite"));
            continue;
        }
        let scale = incoming_total.abs().max(outgoing_total.abs());
        let residual = if scale == 0.0 { 0.0 }
            else { (incoming_total - outgoing_total).abs() / scale };
        if let Some(tolerance) = tolerance {
            if residual > tolerance {
                errors.push(format!("{context} conservation residual {residual:.6e} exceeds tolerance {tolerance:.6e}"));
            }
        }
    }
}

fn resolve_flows<'a>(check: &Value, field: &str, flow_ids: &HashSet<String>,
    by_id: &HashMap<&'a str, &'a Value>, context: &str, errors: &mut Vec<String>)
    -> Vec<(&'a str, &'a Value)> {
    let mut result = Vec::new();
    let Some(values) = check.get(field).and_then(Value::as_array) else {
        errors.push(format!("{context}.{field} must be an array"));
        return result;
    };
    if values.is_empty() {
        errors.push(format!("{context}.{field} must not be empty"));
    }
    let mut seen = HashSet::new();
    for value in values {
        let Some(id) = value.as_str() else {
            errors.push(format!("{context}.{field} entries must be strings"));
            continue;
        };
        if !seen.insert(id) {
            errors.push(format!("{context}.{field} contains duplicate flow {id}"));
        }
        check_ref(Some(id), flow_ids, "flow", &format!("{context}.{field}"), errors);
        if let Some(flow) = by_id.get(id) {
            result.push((id, *flow));
        }
    }
    result
}

fn validate_observables(items: Option<&Vec<Value>>, evidence_ids: &HashSet<String>,
    errors: &mut Vec<String>) {
    for (index, observable) in items.into_iter().flatten().enumerate() {
        check_ref_array(observable, "reference_artifact_ids", evidence_ids,
            "observable reference evidence", &format!("observables[{index}]"),
            false, false, errors);
    }
}

fn validate_qualification(qualification: Option<&Value>, case_family: Option<&str>,
    models: Option<&Vec<Value>>, evidence: &HashSet<String>,
    evidence_by_id: &HashMap<&str, &Value>, errors: &mut Vec<String>) {
    let Some(qualification) = qualification else {
        errors.push("qualification record is required".to_string());
        return;
    };
    check_ref_array(qualification, "supporting_evidence_artifact_ids", evidence,
        "supporting evidence", "qualification", true, false, errors);
    match qualification.get("status").and_then(Value::as_str) {
        Some("qualified") => {
            if qualification.get("independent_review").is_none() {
                errors.push("qualified status requires independent_review".to_string());
            }
            let supporting = qualification.get("supporting_evidence_artifact_ids")
                .and_then(Value::as_array);
            if supporting.is_none_or(Vec::is_empty) {
                errors.push("qualified status requires supporting evidence".to_string());
            }
            if case_family == Some("synthetic_fixture") {
                errors.push("synthetic_fixture cases can never be engineering-qualified".to_string());
            }
            for evidence_id in supporting.into_iter().flatten().filter_map(Value::as_str) {
                if let Some(record) = evidence_by_id.get(evidence_id) {
                    let class = record.get("evidence_class").and_then(Value::as_str);
                    if matches!(class, Some("synthetic_fixture" | "scenario_assumption")) {
                        errors.push(format!(
                            "qualified status cannot rely on {class:?} evidence {evidence_id}"
                        ));
                    }
                }
            }
            for model in models.into_iter().flatten() {
                if model.get("domain_of_validity").and_then(|d| d.get("status"))
                    .and_then(Value::as_str) != Some("in_domain") {
                    errors.push(format!(
                        "qualified status is incompatible with non-in-domain model {}",
                        model.get("id").and_then(Value::as_str).unwrap_or("<missing-id>")
                    ));
                }
            }
        }
        Some("indeterminate") => {
            if qualification.get("blockers").and_then(Value::as_array).is_none_or(Vec::is_empty) {
                errors.push("indeterminate status requires at least one blocker".to_string());
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../benchmarks/conformance/synthetic-power-ledger.case.json"
        )).expect("synthetic fixture JSON")
    }

    #[test]
    fn accepts_synthetic_fixture_and_closes_ledger() {
        let summary = validate_case_manifest(&fixture()).unwrap();
        assert_eq!(summary.case_id, "synthetic-power-ledger-001");
        assert_eq!(summary.model_family, "synthetic_fixture");
        assert_eq!(summary.boundary_count, 5);
        assert_eq!(summary.flow_count, 4);
        assert_eq!(summary.accounting_check_count, 1);
    }

    #[test]
    fn rejects_duplicate_boundary_id() {
        let mut case = fixture();
        let boundaries = case["physical_boundaries"].as_array_mut().unwrap();
        boundaries[1]["id"] = boundaries[0]["id"].clone();
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("duplicate identifier")));
    }

    #[test]
    fn rejects_implicit_model_family_transfer() {
        let mut case = fixture();
        case["model_family"] = Value::String("magnetic_confinement_fusion".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("implicit cross-family")));
    }

    #[test]
    fn rejects_unknown_flow_boundary() {
        let mut case = fixture();
        case["flows"][0]["to_boundary_id"] = Value::String("missing-boundary".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("unknown boundary")));
    }

    #[test]
    fn rejects_non_conserving_power_ledger() {
        let mut case = fixture();
        case["flows"][1]["value"] = Value::from(69.0);
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("conservation residual")));
    }

    #[test]
    fn rejects_validation_holdout_data_leakage() {
        let mut case = fixture();
        case["validation_plan"]["splits"][0]["role"] = Value::String("calibration".into());
        case["validation_plan"]["splits"].as_array_mut().unwrap().push(serde_json::json!({
            "id": "holdout-split", "role": "holdout",
            "dataset_artifact_ids": ["synthetic-ledger-source"]
        }));
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("data leakage")));
    }

    #[test]
    fn rejects_divertor_heat_flux_without_target_surface() {
        let mut case = fixture();
        case["flows"][0]["quantity_kind"] = Value::String("divertor_heat_flux".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("divertor_target_wetted_surface")));
    }

    #[test]
    fn rejects_unknown_uncertainty_reference() {
        let mut case = fixture();
        case["flows"][0]["uncertainty_model_id"] = Value::String("missing-uncertainty".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("unknown uncertainty model")));
    }

    #[test]
    fn rejects_cross_family_case_without_transfer_record() {
        let mut case = fixture();
        case["model_family"] = Value::String("cross_family_contract".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("requires at least one explicit reviewed transfer")));
    }

    #[test]
    fn rejects_cross_family_transfer_with_synthetic_review_evidence() {
        let mut case = fixture();
        case["model_family"] = Value::String("cross_family_contract".into());
        case["models"][0]["model_family"] = Value::String("fusion_plant_systems".into());
        case["models"].as_array_mut().unwrap().push(serde_json::json!({
            "id": "magnetic-plasma-model",
            "name": "Magnetic plasma model",
            "model_family": "magnetic_confinement_fusion",
            "discipline": "plasma_physics",
            "version": "0.1.0",
            "revision": "synthetic-interface-test",
            "domain_of_validity": {
                "summary": "Schema-only negative control.",
                "status": "unknown",
                "known_exclusions": ["No physical validation."]
            },
            "input_artifact_ids": ["synthetic-ledger-source"],
            "output_artifact_ids": []
        }));
        case["cross_family_transfers"] = serde_json::json!([{
            "id": "plant-to-plasma-interface",
            "source_model_id": "toy-ledger-model",
            "destination_model_id": "magnetic-plasma-model",
            "source_family": "fusion_plant_systems",
            "destination_family": "magnetic_confinement_fusion",
            "rationale": "Negative control for a declared model-family interface.",
            "claim": "Schema wiring only; not a physics transfer claim.",
            "independent_review_evidence_id": "synthetic-ledger-source"
        }]);
        let errors = validate_case_manifest(&case).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("independent-review evidence class is insufficient")));
    }

    #[test]
    fn rejects_unknown_evidence_reference() {
        let mut case = fixture();
        case["flows"][0]["evidence_artifact_ids"][0] = Value::String("missing-evidence".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("unknown evidence artifact")));
    }

    #[test]
    fn rejects_unresolved_geometry_evidence() {
        let mut case = fixture();
        case["physical_boundaries"][0]["geometry_artifact_id"] =
            Value::String("missing-geometry".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("unknown geometry evidence")));
    }

    #[test]
    fn rejects_unresolved_observable_reference_evidence() {
        let mut case = fixture();
        case["observables"] = serde_json::json!([{
            "id": "reference-power-check",
            "name": "Reference power",
            "expected_value": 100.0,
            "unit": {"system": "UCUM", "code": "MW"},
            "metric": "absolute_error",
            "tolerance": 1e-6,
            "reference_artifact_ids": ["missing-reference"]
        }]);
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("unknown observable reference evidence")));
    }

    #[test]
    fn rejects_power_quantity_with_energy_unit() {
        let mut case = fixture();
        case["flows"][0]["unit"]["code"] = Value::String("MJ".into());
        assert!(validate_case_manifest(&case).unwrap_err().iter()
            .any(|e| e.contains("incompatible with quantity_kind power")));
    }

    #[test]
    fn rejects_synthetic_case_claiming_engineering_qualification() {
        let mut case = fixture();
        case["qualification"]["status"] = Value::String("qualified".into());
        case["qualification"]["independent_review"] = serde_json::json!({
            "review_process": "negative-control",
            "reviewer_or_body": "test-harness",
            "reviewed_on": "2026-11-18",
            "scope": "synthetic fixture only"
        });
        let errors = validate_case_manifest(&case).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("synthetic_fixture cases can never be engineering-qualified")));
        assert!(errors.iter().any(|e| e.contains("cannot rely on")));
    }
}
