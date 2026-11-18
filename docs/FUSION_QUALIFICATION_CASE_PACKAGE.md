# Fusion qualification case package

**Status:** proposed, schema-backed v0.1  
**Tracking:** [Issue #39](https://github.com/Luminous-Dynamics/spark-engine/issues/39)  
**Related work:** [power-exhaust boundary contract](POWER_EXHAUST_BOUNDARY_CONTRACT.md) (in the current research branch), [exhaust benchmark corpus #33](https://github.com/Luminous-Dynamics/spark-engine/issues/33), [RAMI #34](https://github.com/Luminous-Dynamics/spark-engine/issues/34), [whole-plant net electricity #35](https://github.com/Luminous-Dynamics/spark-engine/issues/35), [robust co-design #36](https://github.com/Luminous-Dynamics/spark-engine/issues/36), [transient component lifetime #38](https://github.com/Luminous-Dynamics/spark-engine/issues/38), and [tritium fuel-cycle closure #37](https://github.com/Luminous-Dynamics/spark-engine/issues/37), plus [model-family separation #41](https://github.com/Luminous-Dynamics/spark-engine/issues/41) and [fusion-neutronics benchmarks #42](https://github.com/Luminous-Dynamics/spark-engine/issues/42).

This document defines a proposed, code-agnostic package for carrying a fusion engineering case from source data through model execution, comparison and qualification. It is an interoperability contract, not a plasma, neutronics, thermal-hydraulic, material or finance solver. The schema is a first implementation artifact; the semantic validator, canonical digest tooling and domain adapters remain follow-on work.

## 1. Why a case package is needed

Fusion plant design combines models with different physical boundaries, temporal resolutions, assumptions, units, validation domains and evidence sources. A result can be numerically reproducible and still be physically inapplicable. A source can be cited but fail to support the value attributed to it. A heat-flux number can be dimensionally valid but normalized to the wrong surface. A model can interpolate well within its calibration set and fail under a new device or operating regime.

A useful case package must therefore preserve more than input and output values. It must preserve the *meaning and lineage* of those values:

- Which physical boundary does a quantity cross or describe?
- Is it a surface average, local peak, volumetric source, integrated amount or rate?
- Over what time interval and at what temporal resolution?
- Which model, executable, configuration and geometry produced it?
- What source supports each input, and under what licence can it be reused?
- Which uncertainties are aleatory, epistemic or due to model discrepancy?
- Which data were used for calibration, validation and final holdout?
- What was actually checked, by whom, and what remains unknown?

The package is intended to let specialist codes remain specialist codes while making their exchanges inspectable and repeatable.

### 1.1 Model-family isolation is mandatory

The case manifest and each model record require explicit `model_family` declarations. The initial controlled vocabulary distinguishes synthetic fixtures, lattice-confinement-fusion research, magnetic-confinement fusion, inertial-confinement fusion, magnetized-target fusion, generic fusion-plant systems work and explicitly reviewed cross-family contracts. Every model in a case must declare its own family; a case whose family is `cross_family_contract` requires an explicit transfer contract and independent review in the semantic-validation layer.

This distinction protects a critical scientific boundary in Spark Engine: its original LCF anomaly-investigation programme is not a validated fusion power-plant model. The Q_econ and plant-systems work adapts a published economic framework and depends on separately qualified conventional-fusion physics and engineering inputs. Those programmes may share units, provenance, uncertainty and evidence-package infrastructure, but must not silently share physical priors, calibration data or qualification status.

A cross-family transfer must identify both source and destination model families, declare the physical argument for transfer, retain separate provenance and uncertainty, and pass an explicit independent review. The schema's family identifier is a guardrail against accidental mixing; it does not prove the family assignment or validate a model.

See [Issue #41](https://github.com/Luminous-Dynamics/spark-engine/issues/41). This is not an abandonment of LCF research. It preserves the integrity of both research programmes and prevents a result in one domain from being misrepresented as evidence in another.

## 2. Standards-first packaging

The proposed package uses existing formats where possible:

1. **JSON Schema 2020-12** validates the domain manifest's structure and required fields. It does not validate physics or dynamic references between every item.
2. **RO-Crate 1.3** packages the manifest with datasets, geometry, model files, scripts, solver outputs, reports and provenance metadata. RO-Crate describes research data and how resources were created and can be reused.
3. **RFC 8785 JSON Canonicalization Scheme (JCS)** provides deterministic canonical JSON bytes for hashing. The digest is an identity/integrity receipt, not a truth claim.
4. A signed attestation format such as **DSSE/in-toto** may be evaluated for execution attestations. Do not create a bespoke cryptographic envelope before checking interoperability and threat requirements.

Suggested directory layout:

```text
case-package/
  ro-crate-metadata.json
  case-manifest.json
  schema/
    fusion-qualification-case.schema.json
  evidence/
    sources/
    measurements/
    geometry/
  models/
    manifests/
    configurations/
  runs/
    inputs/
    outputs/
    logs/
    attestations/
  validation/
    splits/
    comparisons/
    reports/
```

The RO-Crate metadata document describes the crate and files. The case manifest is the domain-specific record validated by the JSON Schema. Large or licensed artefacts may be referenced by stable URI and digest rather than copied into the package.

## 3. Physical-boundary contract

A system boundary is a named physical interface, not merely a diagram label. Each boundary has a stable identifier, domain, role and optional geometry artefact. A directed flow connects two declared boundaries and carries a quantity, numerical value, unit, time basis, evidence references and optional uncertainty reference.

The package must distinguish, at minimum, where relevant:

- fusion reaction / plasma source;
- confined plasma and separatrix;
- scrape-off layer and radiation sinks;
- energy-extraction surface;
- first wall and blanket surfaces;
- each divertor target's wetted surface;
- coolant and heat-exchanger boundaries;
- electrical bus, recirculating loads and grid connection;
- tritium-bearing subsystem control volumes and facility transfer interfaces;
- replaceable components, remote-handling interfaces and maintenance resources.

A common unit is not a common boundary. For example, Q_econ's fusion power density on an energy-extraction surface must not be substituted for divertor target heat flux on a wetted target surface. The mapping requires a declared power ledger, geometry and model inputs; this package records the mapping and its provenance but does not validate the upstream physics.

The schema intentionally makes time basis explicit. A steady-state average, a finite-window average, a sampled time history and a campaign/lifetime total are not interchangeable.

### 3.1 Semantic validation beyond JSON Schema

A second validation stage must check cross-record invariants that JSON Schema alone cannot reliably express:

- every boundary, time-basis, uncertainty and evidence reference resolves;
- boundary and flow identifiers are unique;
- a flow's endpoints exist and are not silently reversed;
- units are compatible with the declared quantity;
- conversion between units is explicit and recorded;
- average, peak, integrated and instantaneous quantities are never silently substituted;
- all required conservation ledgers close within declared tolerances;
- power, mass and tritium accounts do not double-count internal transfers;
- source, model and geometry digests correspond to the actual packaged objects;
- calibration, validation and holdout partitions do not overlap under the declared grouping rule;
- model use is within its declared domain, or the result is marked out-of-domain / indeterminate.

No missing quantity is filled with zero. Unknown is a valid state with a reason and a named owner for resolution.

## 4. Evidence semantics

Evidence classes describe the origin or comparison mode of an artefact. They are deliberately **not a single ordered ladder**: a measurement, a published report and a code-to-code comparison answer different questions. One must not be automatically upgraded to another.

The v0.1 schema recognizes:

| Class | Meaning | Does not establish by itself |
|---|---|---|
| `synthetic_fixture` | Constructed data for software or method testing | Physical behavior |
| `scenario_assumption` | Explicit hypothetical design input | Likelihood or empirical support |
| `literature_reported` | Value or result transcribed from a publication | Reproduction or transfer to another concept |
| `measured` | Data acquired by an identified measurement process | Measurement validity, representativeness or model agreement |
| `code_to_code_comparison` | Comparison between separately identified implementations | Agreement with nature |
| `experimental_validation` | Model comparison against identified experimental observations | Validity outside tested conditions |
| `engineering_qualification` | Evidence supporting a declared component/system requirement | Qualification beyond stated scope and configuration |

Every evidence artefact must state the exact claim it supports, its scope, producer, locator, licence, retrieval date where relevant and content digest. A source URL is a locator, not proof that the source supports the attributed value. The package must retain negative results, failed cases, exclusions and unresolved provenance.

## 5. Uncertainty and applicability

The package distinguishes three uncertainty categories:

- **Aleatory uncertainty:** variability represented as part of the modeled population or operating process.
- **Epistemic uncertainty:** lack of knowledge about parameters, states or mechanisms.
- **Model discrepancy:** systematic difference between the model and the physical system or a higher-fidelity reference.

The category must be declared rather than inferred from a distribution shape. A Gaussian or triangular distribution does not, on its own, establish that uncertainty is aleatory or calibrated. Dependence assumptions are explicit; independent sampling is not a safe default for physically coupled variables.

Each model declares a domain of validity: device/concept family, operating regime, geometry/material range, temporal resolution, calibration support and known exclusions. The semantic validator must not convert an out-of-domain result into a nominal prediction.

Calibration, validation and holdout partitions are frozen before tuning. The split key should reflect the generalization question: device, campaign, operating regime, geometry family, material batch or time period. Random row-level splits are inadequate when correlated shots or adjacent time windows leak information across partitions.

## 6. Qualification outcomes

A case outcome is one of:

- **indeterminate:** required input, provenance, model domain or independent evidence is missing;
- **not_qualified:** the available evidence does not satisfy the declared qualification criteria;
- **qualified:** all declared mandatory gates passed for the exact configuration and scope;
- **rejected:** a declared hard constraint failed;
- **out_of_domain:** one or more critical models were used outside their supported domain.

A result is qualified only for the exact package digest, model/configuration revisions, geometry, operating envelope, time basis and evidence scope reviewed. A qualified result must identify an independent reviewer or review process and the evidence supporting each mandatory gate. Re-running the same code, signing an attestation or passing CI does not independently qualify physical behavior.

The package should report separate dimensions instead of one opaque confidence score: software verification, numerical verification, model validation, engineering qualification and operational evidence.

## 7. First conformance fixture

The included `synthetic-power-ledger.case.json` is intentionally artificial. It describes a simple 100 MW input split into 70 MW, 20 MW and 10 MW outgoing channels over a declared one-second window. Its purpose is to test required fields, boundary references and a conservation observable. It is not a fusion design, not a divertor operating point and not validation data.

The corresponding schema should accept the positive fixture and later be paired with negative fixtures for:

- unknown boundary or evidence identifiers;
- duplicate IDs;
- missing unit or time basis;
- flow with incompatible quantity/unit;
- target heat flux joined to an energy-extraction area;
- unlabelled time-averaged value used as a transient peak;
- unknown silently encoded as zero;
- out-of-domain model marked qualified;
- holdout data reused for calibration;
- tampered content with unchanged digest.

## 8. Benchmark and conformance targets

These are proposed engineering targets, not achieved results or community-standard limits.

| Gate | Test | Proposed target |
|---|---|---|
| S0 | Schema fixtures | 100% of authored positive and negative fixtures classified correctly |
| S1 | Referential integrity | Zero unresolved or duplicate identifiers in accepted packages |
| S2 | Canonicalization | Two independent JCS implementations produce identical canonical bytes and SHA-256 |
| S3 | Unit/time/boundary semantics | Zero implicit unit, surface-role or time-basis coercions |
| S4 | Ledger invariants | Relative residual ≤ 1e-10 on analytic fixtures, with tolerance and conditioning reported |
| S5 | Replay | Deterministic fixtures reproduce outputs from frozen inputs and toolchain |
| S6 | Benchmark independence | Holdout split frozen before calibration; leakage negative controls rejected |
| S7 | Physical validation | Report bias, RMSE/NRMSE, tail error and interval coverage on independent references |
| S8 | Qualification safety | Zero false-safe classifications in the frozen corpus, with sample count and one-sided confidence bound |

A zero-failure result without its sample count is not informative. With zero false-safe results in (n) independent cases, report a one-sided binomial confidence bound; do not translate a small test corpus into a general safety claim.

## 9. Mapping to the Luminous projects

| Project | Role in the package |
|---|---|
| Spark Engine | Fusion-specific scenario, Q_econ, exhaust, plant energy, economics and orchestration manifests |
| Symtropy | Geometry, thermal/structural response, remote-handling and operational simulation receipts |
| Symthaea | Hypothesis generation, causal analysis, discrepancy attribution and value-of-information proposals; never self-certification |
| Mycelix | Provenance, immutable evidence bundles, independent review receipts and authorization boundaries |
| Sol-Atlas | Spatial component, material, radiation, energy and maintenance evidence layers |

This is an adapter architecture, not a demand that every project adopt one monolithic internal data model. Each subsystem owns its physical model and can publish a package or adapter that conforms to the shared boundary/evidence contract.

## 10. Relation to existing work and industry

The direction complements existing specialist systems:

- **PROCESS** is a self-consistent fusion systems code that couples plant physics and engineering constraints and can optimize a user-selected figure of merit. Its public documentation describes heat conversion and divertor models. The qualification package should compare to PROCESS where boundaries and conventions can be matched, not claim to replace it.
- **SOLPS-ITER** and STEP exhaust workflows provide higher-fidelity/reduced-model comparisons for plasma-edge and target-load behavior. An adapter must preserve target-profile, flux, geometry and time conventions.
- **OpenMC Fusion Benchmarks** is a strong precedent for schema-driven, code-agnostic fusion benchmarking, including CAD geometry, standardized case specifications and automated comparisons. It is a neutronics benchmark framework; reuse its schema/reproducibility pattern, not its physical data as divertor evidence.
- **DOE's June 2026 Fusion Science and Technology Roadmap** calls for FAIR-oriented data infrastructure, interoperable whole-plant VV/UQ, model validity domains, quantified uncertainty, RAMI metrics, remote-maintenance standards, tritium fuel-cycle milestones and early identification of design killers. The roadmap validates the importance of this programme direction; it does not validate this implementation.

The differentiator should be reproducible model-to-model contracts, evidence lineage and fail-closed qualification decisions, rather than a claim to have superior facility access, experimental datasets or specialist physics solvers.

## 11.1 First semantic conformance implementation

The branch includes a development-only Python validator at `scripts/validate_fusion_cases.py` and a focused GitHub Actions workflow at `.github/workflows/fusion-case-package.yml`. Python is used only as a CI/test harness; it is not a runtime or production dependency of Spark Engine.

The validator:
- validates the manifest with JSON Schema Draft 2020-12 and a format checker;
- checks uniqueness and referential integrity for boundaries, time bases, flows, models, uncertainty records and evidence artefacts;
- evaluates explicitly declared conservation ledgers using exact unit/time-basis matches, directed boundary joins and relative residual tolerances; it does not assume every boundary must close without a declared check;
- checks a deliberately bounded UCUM unit profile for common power, energy, heat-flux, mass, mass-flow, temperature and pressure quantities;
- requires fusion power density to name the energy-extraction surface and divertor heat flux to name a divertor target wetted surface, including the declared spatial statistic;
- rejects unresolved time-series artefact references and calibration/validation/holdout split overlap;
- checks local evidence SHA-256 digests against the actual packaged files;
- enforces model-family isolation and explicit, independently evidenced transfers for cross-family contracts;
- prevents synthetic fixtures or scenario assumptions from qualifying a physical case;
- executes negative controls for family contamination, holdout leakage, synthetic false qualification and mismatched surface-role joins.

This is a first conformance profile, not a general UCUM implementation, full RO-Crate validator, JCS canonicalizer, signature verifier or physical model validator. It deliberately fails closed for unsupported or ambiguous combinations. Expanding the unit vocabulary should be done with tested UCUM-compatible semantics, not string guessing.

Run locally with Python 3.12 and `python -m pip install 'jsonschema[format]==4.26.0'`, then `python scripts/validate_fusion_cases.py`. The dedicated CI job must still pass on the exact PR head before the validator is called CI-validated.

## 12. Implementation sequence

1. Validate this schema and the synthetic fixture with at least two independent JSON Schema implementations.
2. Add negative conformance fixtures and a semantic reference checker for IDs, units, time bases and boundary roles.
3. Add a canonicalization tool using RFC 8785 test vectors; hash the manifest without its integrity/attestation envelope to avoid recursive hashing.
4. Package one Q_econ scenario and one power-exhaust bridge case as a RO-Crate, retaining distinct extraction and target surfaces.
5. Add PROCESS comparison cases with frozen versions, inputs, boundary conventions and outputs.
6. Add public STEP/SOLPS-ITER, thermal/material, RAMI and tritium fuel-cycle cases only when redistribution rights and provenance are clear.
7. Connect independent validation receipts and exact package digests to Mycelix.

## 13. References

- U.S. DOE, [Fusion Science and Technology Roadmap (June 2026)](https://www.energy.gov/documents/fusion-science-and-technology-roadmap), especially whole-plant VV/UQ, FAIR data, RAMI, remote maintenance, blanket and tritium fuel-cycle milestones.
- UKAEA, [PROCESS overview](https://www.ukaea.org/service/process/) and [public documentation](https://ukaea.github.io/PROCESS/).
- UKAEA, [PROCESS divertor model](https://ukaea.github.io/PROCESS/eng-models/divertor/).
- OpenMC Fusion Benchmarks, [introduction](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/intro.html) and [benchmark schema](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/specifications/schema.html).
- [RO-Crate 1.3 specification](https://www.researchobject.org/ro-crate/specification).
- IETF, [RFC 8785: JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html).
- STEP collaborators, [divertor design and reduced models](https://doi.org/10.1088/1741-4326/ad93e7).
- Tesini et al., [DEMO remote maintenance development](https://www.sciencedirect.com/science/article/pii/S0920379622001211).

---

**Current qualification status:** schema and documentation proposal only. No integrated semantic validator, canonical hash/signature pipeline, external code comparison, experimental validation or plant qualification is claimed by this document.
