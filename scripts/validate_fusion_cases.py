#!/usr/bin/env python3
"""Development-only conformance checks for fusion qualification case packages.

This is not production fusion software and does not validate physical models.
It validates JSON structure, referential integrity, selected boundary semantics,
local evidence digests, split separation and model-family isolation.
"""

from __future__ import annotations

import copy
import hashlib
import json
import math
import sys
from pathlib import Path
from urllib.parse import urlparse

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "fusion-qualification-case.schema.json"
FIXTURE_DIR = ROOT / "benchmarks" / "conformance"

# UCUM codes accepted by this initial conformance profile. This is deliberately
# a small explicit subset, not a general UCUM parser or unit-conversion engine.
UNIT_CODES = {
    "power": {"W", "kW", "MW", "GW"},
    "electric_power": {"W", "kW", "MW", "GW"},
    "energy": {"J", "kJ", "MJ", "GJ", "Wh", "kWh", "MWh", "GWh"},
    "electrical_energy": {"J", "kJ", "MJ", "GJ", "Wh", "kWh", "MWh", "GWh"},
    "fusion_power_density": {"W.m-2", "kW.m-2", "MW.m-2"},
    "divertor_heat_flux": {"W.m-2", "kW.m-2", "MW.m-2"},
    "heat_flux": {"W.m-2", "kW.m-2", "MW.m-2"},
    "mass": {"g", "kg"},
    "mass_flow": {"g.s-1", "kg.s-1"},
    "tritium_flow": {"g.s-1", "kg.s-1"},
    "particle_rate": {"s-1"},
    "temperature": {"K", "Cel"},
    "pressure": {"Pa", "kPa", "MPa", "bar"},
}


def _unique_ids(items: list[dict], label: str, errors: list[str]) -> set[str]:
    seen: set[str] = set()
    for index, item in enumerate(items):
        identifier = item.get("id")
        if not isinstance(identifier, str):
            errors.append(f"{label}[{index}] has no string id")
        elif identifier in seen:
            errors.append(f"duplicate {label} id: {identifier}")
        else:
            seen.add(identifier)
    return seen


def _check_refs(
    values: list[str],
    available: set[str],
    label: str,
    errors: list[str],
) -> None:
    for value in values:
        if value not in available:
            errors.append(f"{label} references unknown id: {value}")


def _local_locator_path(locator: str) -> Path | None:
    parsed = urlparse(locator)
    if parsed.scheme or Path(locator).is_absolute():
        return None
    candidate = (ROOT / locator).resolve()
    try:
        candidate.relative_to(ROOT.resolve())
    except ValueError:
        return None
    return candidate


def validate_manifest(
    manifest: dict,
    schema: dict,
    *,
    verify_local_digests: bool = True,
) -> list[str]:
    errors: list[str] = []
    validator = Draft202012Validator(schema, format_checker=FormatChecker())
    for error in sorted(validator.iter_errors(manifest), key=lambda e: list(map(str, e.absolute_path))):
        location = ".".join(map(str, error.absolute_path)) or "<root>"
        errors.append(f"schema {location}: {error.message}")
    if errors:
        return errors

    boundaries = manifest["physical_boundaries"]
    time_bases = manifest["time_bases"]
    flows = manifest["flows"]
    quantities = manifest["quantities"]
    models = manifest["models"]
    uncertainty_models = manifest["uncertainty_models"]
    evidence = manifest["evidence_artifacts"]

    boundary_ids = _unique_ids(boundaries, "physical_boundaries", errors)
    time_ids = _unique_ids(time_bases, "time_bases", errors)
    flow_ids = _unique_ids(flows, "flows", errors)
    _ = flow_ids
    _unique_ids(quantities, "quantities", errors)
    model_ids = _unique_ids(models, "models", errors)
    uncertainty_ids = _unique_ids(uncertainty_models, "uncertainty_models", errors)
    evidence_ids = _unique_ids(evidence, "evidence_artifacts", errors)

    boundary_by_id = {item["id"]: item for item in boundaries}
    time_by_id = {item["id"]: item for item in time_bases}
    evidence_by_id = {item["id"]: item for item in evidence}
    model_by_id = {item["id"]: item for item in models}

    for flow in flows:
        _check_refs(
            [flow["from_boundary_id"], flow["to_boundary_id"]],
            boundary_ids,
            f"flow {flow['id']} boundary",
            errors,
        )
        _check_refs([flow["time_basis_id"]], time_ids, f"flow {flow['id']} time basis", errors)
        _check_refs(
            flow["evidence_artifact_ids"],
            evidence_ids,
            f"flow {flow['id']} evidence",
            errors,
        )
        uncertainty_id = flow.get("uncertainty_model_id")
        if uncertainty_id is not None and uncertainty_id not in uncertainty_ids:
            errors.append(f"flow {flow['id']} references unknown uncertainty model: {uncertainty_id}")

        quantity_kind = flow["quantity_kind"]
        unit_code = flow["unit"]["code"]
        allowed_units = UNIT_CODES.get(quantity_kind)
        if allowed_units is not None and unit_code not in allowed_units:
            errors.append(
                f"flow {flow['id']} unit {unit_code!r} is not in the declared "
                f"{quantity_kind} unit profile"
            )

        if quantity_kind in {"heat_flux", "fusion_power_density", "divertor_heat_flux"}:
            surface_role = flow["surface_role"]
            endpoint_roles = {
                boundary_by_id[boundary_id]["role"]
                for boundary_id in (flow["from_boundary_id"], flow["to_boundary_id"])
                if boundary_id in boundary_by_id
            }
            if surface_role not in endpoint_roles:
                errors.append(
                    f"flow {flow['id']} surface_role {surface_role!r} is not "
                    "the role of either declared endpoint boundary"
                )
        elif "surface_role" in flow or "spatial_statistic" in flow:
            errors.append(
                f"flow {flow['id']} assigns surface semantics to a non-surface quantity"
            )

        time_basis = time_by_id.get(flow["time_basis_id"])
        if time_basis and time_basis["kind"] == "time_series":
            artifact_id = flow.get("time_series_artifact_id")
            if not artifact_id:
                errors.append(
                    f"flow {flow['id']} uses time_series but has no time_series_artifact_id"
                )
            elif artifact_id not in evidence_ids:
                errors.append(
                    f"flow {flow['id']} references unknown time-series artifact: {artifact_id}"
                )

    # Explicit conservation checks are caller-declared: the validator
    # never assumes every boundary must conserve a scalar independently.
    accounting_checks = manifest.get("accounting_checks", [])
    _unique_ids(accounting_checks, "accounting_checks", errors)
    flow_by_id = {flow["id"]: flow for flow in flows}
    for check in accounting_checks:
        boundary_id = check["boundary_id"]
        if boundary_id not in boundary_ids:
            errors.append(
                f"accounting check {check['id']} references unknown boundary: {boundary_id}"
            )
        _check_refs(
            [check["time_basis_id"]],
            time_ids,
            f"accounting check {check['id']} time basis",
            errors,
        )
        _check_refs(
            check["evidence_artifact_ids"],
            evidence_ids,
            f"accounting check {check['id']} evidence",
            errors,
        )
        inbound_ids = check["inbound_flow_ids"]
        outbound_ids = check["outbound_flow_ids"]
        overlap = set(inbound_ids).intersection(outbound_ids)
        if overlap:
            errors.append(
                f"accounting check {check['id']} reuses flows in both directions: "
                + ", ".join(sorted(overlap))
            )
        selected_ids = inbound_ids + outbound_ids
        _check_refs(
            selected_ids,
            set(flow_by_id),
            f"accounting check {check['id']} flow",
            errors,
        )
        selected = [flow_by_id[item] for item in selected_ids if item in flow_by_id]
        valid = len(selected) == len(selected_ids)
        for flow in selected:
            if flow["quantity_kind"] != check["quantity_kind"]:
                errors.append(
                    f"accounting check {check['id']} quantity kind differs from flow {flow['id']}"
                )
                valid = False
            if flow["unit"] != check["unit"]:
                errors.append(
                    f"accounting check {check['id']} unit differs from flow {flow['id']}; "
                    "implicit unit conversion is not permitted"
                )
                valid = False
            if flow["time_basis_id"] != check["time_basis_id"]:
                errors.append(
                    f"accounting check {check['id']} time basis differs from flow {flow['id']}"
                )
                valid = False
            if flow["value"] < 0.0:
                errors.append(
                    f"accounting check {check['id']} requires non-negative directed flow values"
                )
                valid = False
        for flow_id in inbound_ids:
            flow = flow_by_id.get(flow_id)
            if flow is not None and flow["to_boundary_id"] != boundary_id:
                errors.append(
                    f"accounting check {check['id']} inbound flow {flow_id} "
                    "does not terminate at its declared boundary"
                )
                valid = False
        for flow_id in outbound_ids:
            flow = flow_by_id.get(flow_id)
            if flow is not None and flow["from_boundary_id"] != boundary_id:
                errors.append(
                    f"accounting check {check['id']} outbound flow {flow_id} "
                    "does not originate at its declared boundary"
                )
                valid = False
        if valid:
            inbound_total = math.fsum(flow_by_id[item]["value"] for item in inbound_ids)
            outbound_total = math.fsum(flow_by_id[item]["value"] for item in outbound_ids)
            denominator = max(abs(inbound_total), abs(outbound_total), 1e-300)
            relative_residual = abs(inbound_total - outbound_total) / denominator
            if relative_residual > check["relative_tolerance"]:
                errors.append(
                    f"accounting check {check['id']} relative residual "
                    f"{relative_residual:.12g} exceeds tolerance "
                    f"{check['relative_tolerance']:.12g}"
                )

    for quantity in quantities:
        boundary_id = quantity.get("physical_boundary_id")
        if boundary_id is not None and boundary_id not in boundary_ids:
            errors.append(
                f"quantity {quantity['id']} references unknown physical boundary: {boundary_id}"
            )
        time_id = quantity.get("time_basis_id")
        if time_id is not None and time_id not in time_ids:
            errors.append(f"quantity {quantity['id']} references unknown time basis: {time_id}")
        _check_refs(
            quantity.get("evidence_artifact_ids", []),
            evidence_ids,
            f"quantity {quantity['id']} evidence",
            errors,
        )
        uncertainty_id = quantity.get("uncertainty_model_id")
        if uncertainty_id is not None and uncertainty_id not in uncertainty_ids:
            errors.append(
                f"quantity {quantity['id']} references unknown uncertainty model: {uncertainty_id}"
            )

    for boundary in boundaries:
        geometry_id = boundary.get("geometry_artifact_id")
        if geometry_id is not None and geometry_id not in evidence_ids:
            errors.append(
                f"boundary {boundary['id']} references unknown geometry artifact: {geometry_id}"
            )

    for uncertainty in uncertainty_models:
        _check_refs(
            uncertainty.get("source_artifact_ids", []),
            evidence_ids,
            f"uncertainty {uncertainty['id']} source",
            errors,
        )

    for model in models:
        _check_refs(
            model.get("input_artifact_ids", []),
            evidence_ids,
            f"model {model['id']} input",
            errors,
        )
        _check_refs(
            model.get("output_artifact_ids", []),
            evidence_ids,
            f"model {model['id']} output",
            errors,
        )

    validation_splits = manifest["validation_plan"]["splits"]
    _unique_ids(validation_splits, "validation_plan.splits", errors)
    seen_dataset_ids: set[str] = set()
    for split in validation_splits:
        dataset_ids = set(split["dataset_artifact_ids"])
        _check_refs(
            list(dataset_ids),
            evidence_ids,
            f"split {split['id']} dataset",
            errors,
        )
        overlap = seen_dataset_ids.intersection(dataset_ids)
        for artifact_id in sorted(overlap):
            errors.append(
                f"validation data leakage: artifact {artifact_id} appears in multiple splits"
            )
        seen_dataset_ids.update(dataset_ids)

    for observable in manifest["observables"]:
        _check_refs(
            observable.get("reference_artifact_ids", []),
            evidence_ids,
            f"observable {observable['id']} reference",
            errors,
        )

    qualification = manifest["qualification"]
    _check_refs(
        qualification["supporting_evidence_artifact_ids"],
        evidence_ids,
        "qualification supporting evidence",
        errors,
    )
    if qualification["status"] == "qualified":
        if any(
            model["domain_of_validity"]["status"] != "in_domain"
            for model in models
        ):
            errors.append("qualified status is incompatible with an out-of-domain or unknown model")
        for evidence_id in qualification["supporting_evidence_artifact_ids"]:
            record = evidence_by_id.get(evidence_id)
            if record and record["evidence_class"] in {"synthetic_fixture", "scenario_assumption"}:
                errors.append(
                    f"qualified status cannot rely on {record['evidence_class']} evidence: {evidence_id}"
                )

    case_family = manifest["model_family"]
    model_families = {model["model_family"] for model in models}
    if case_family != "cross_family_contract":
        for model in models:
            if model["model_family"] != case_family:
                errors.append(
                    f"model-family contamination: case is {case_family}, "
                    f"model {model['id']} is {model['model_family']}"
                )
    else:
        if len(model_families) < 2:
            errors.append("cross_family_contract requires at least two distinct model families")
        transfers = manifest.get("cross_family_transfers", [])
        if not transfers:
            errors.append("cross_family_contract requires explicit cross_family_transfers")
        _unique_ids(transfers, "cross_family_transfers", errors)
        covered_pairs: set[tuple[str, str]] = set()
        for transfer in transfers:
            source = model_by_id.get(transfer["source_model_id"])
            destination = model_by_id.get(transfer["destination_model_id"])
            if source is None or destination is None:
                errors.append(f"cross-family transfer {transfer['id']} references an unknown model")
                continue
            if source["model_family"] != transfer["source_family"]:
                errors.append(f"cross-family transfer {transfer['id']} source family mismatch")
            if destination["model_family"] != transfer["destination_family"]:
                errors.append(f"cross-family transfer {transfer['id']} destination family mismatch")
            if transfer["source_family"] == transfer["destination_family"]:
                errors.append(f"cross-family transfer {transfer['id']} does not cross model families")
            review_id = transfer["independent_review_evidence_id"]
            if review_id not in evidence_ids:
                errors.append(f"cross-family transfer {transfer['id']} lacks a review evidence record")
            else:
                review = evidence_by_id[review_id]
                if review["evidence_class"] in {"synthetic_fixture", "scenario_assumption"}:
                    errors.append(
                        f"cross-family transfer {transfer['id']} review evidence is not independent"
                    )
            covered_pairs.add((transfer["source_model_id"], transfer["destination_model_id"]))
        for source in models:
            for destination in models:
                if source["model_family"] != destination["model_family"]:
                    if (source["id"], destination["id"]) not in covered_pairs:
                        errors.append(
                            "missing explicit transfer contract for "
                            f"{source['id']} -> {destination['id']}"
                        )

    if case_family == "synthetic_fixture" and qualification["status"] == "qualified":
        errors.append("synthetic_fixture cases can never be engineering-qualified")

    if verify_local_digests:
        for record in evidence:
            local_path = _local_locator_path(record["locator"])
            if local_path is None:
                continue
            if not local_path.is_file():
                errors.append(f"local evidence file is missing: {record['locator']}")
                continue
            actual = hashlib.sha256(local_path.read_bytes()).hexdigest()
            expected = record["content_digest_sha256"].lower()
            if actual != expected:
                errors.append(
                    f"evidence digest mismatch for {record['id']}: expected {expected}, got {actual}"
                )

    return errors


def main() -> int:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema, format_checker=FormatChecker())

    fixtures = sorted(FIXTURE_DIR.glob("*.case.json"))
    if not fixtures:
        print("No *.case.json conformance fixtures found.", file=sys.stderr)
        return 2

    loaded: list[tuple[Path, dict]] = []
    for path in fixtures:
        manifest = json.loads(path.read_text(encoding="utf-8"))
        errors = validate_manifest(manifest, schema)
        if errors:
            print(f"FAIL {path.relative_to(ROOT)}")
            for error in errors:
                print(f"  - {error}")
            return 1
        loaded.append((path, manifest))
        print(f"PASS {path.relative_to(ROOT)}")

    # Negative controls ensure that key unsafe states fail closed.
    _, positive = loaded[0]
    controls: list[tuple[str, dict]] = []

    wrong_family = copy.deepcopy(positive)
    wrong_family["model_family"] = "magnetic_confinement_fusion"
    controls.append(("cross-family mixing", wrong_family))

    leaked_holdout = copy.deepcopy(positive)
    leaked_holdout["validation_plan"]["splits"].append({
        "id": "deliberate-leak-control",
        "role": "calibration",
        "dataset_artifact_ids": ["synthetic-ledger-source"],
    })
    controls.append(("validation split leakage", leaked_holdout))

    false_qualification = copy.deepcopy(positive)
    false_qualification["qualification"]["status"] = "qualified"
    false_qualification["qualification"]["decision_basis"] = "Negative control only."
    false_qualification["qualification"]["independent_review"] = {
        "review_process": "negative-control",
        "reviewer_or_body": "test-harness",
        "reviewed_on": "2026-11-18",
        "scope": "synthetic negative control",
    }
    controls.append(("synthetic false qualification", false_qualification))

    for label, candidate in controls:
        if not validate_manifest(candidate, schema):
            print(f"FAIL negative control did not reject: {label}", file=sys.stderr)
            return 1
        print(f"PASS rejected negative control: {label}")

    # A balanced ledger that is tampered by one MW must fail its declared closure check.
    imbalanced_ledger = copy.deepcopy(positive)
    sink_c = next(flow for flow in imbalanced_ledger["flows"] if flow["id"] == "sink-c-power")
    sink_c["value"] += 1.0
    if not validate_manifest(imbalanced_ledger, schema):
        print("FAIL negative control did not reject: non-closing power ledger", file=sys.stderr)
        return 1
    print("PASS rejected negative control: non-closing power ledger")

    # A heat-flux value with a valid unit but a surface role not present on
    # either endpoint must be rejected by the semantic boundary join.
    boundary_mismatch = copy.deepcopy(positive)
    boundary_mismatch["flows"][0]["quantity_kind"] = "divertor_heat_flux"
    boundary_mismatch["flows"][0]["surface_role"] = "divertor_target_wetted_surface"
    boundary_mismatch["flows"][0]["spatial_statistic"] = "surface_average"
    boundary_mismatch["flows"][0]["unit"]["code"] = "MW.m-2"
    if not validate_manifest(boundary_mismatch, schema):
        print("FAIL negative control did not reject: mismatched surface role", file=sys.stderr)
        return 1
    print("PASS rejected negative control: mismatched surface role")

    print(f"Validated {len(loaded)} fixture(s) and {len(controls) + 2} negative control(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
