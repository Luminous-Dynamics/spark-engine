# Fusion architecture decision-stability benchmark — 2026

**Status:** four synthetic controls and targeted Rust known-answer tests are authored and wired; exact-head CI/test success and any performance result are not claimed  
**Tracking:** [FUSION-DECISION-ROBUSTNESS-001, Spark #44](https://github.com/Luminous-Dynamics/spark-engine/issues/44)  
**Composes with:** [FUSION-E2E-001 #43](https://github.com/Luminous-Dynamics/spark-engine/issues/43), [Spark robust co-design #36](https://github.com/Luminous-Dynamics/spark-engine/issues/36), and the existing Symthaea/Symtropy UQ and model-discrepancy owners below.

## 1. Thesis: optimize the stability of the decision, not just the design point

A nominal optimizer finds the best candidate under one declared model and input set. A robust engineering programme must also determine whether that recommendation survives:
- defensible alternative physics formulations;
- correlated parameter uncertainty;
- measured model-form discrepancy;
- applicability and regime shifts;
- operating and grid scenarios;
- maintenance and lifecycle constraints;
- declared changes in stakeholder preference.

The new output is a **decision atlas**: a source-linked map of which candidate is feasible, dominated, indeterminate or preferred under each qualified model/scenario combination, and the assumptions that change the conclusion.

It is not a new optimizer or uncertainty estimator. It composes the owners already being built.

## 2. Why this matters against current fusion practice

### 2.1 Systems-code disagreement can be decision-relevant

Ashe et al., *Benchmarking of spherical tokamak power plant design in PROCESS and SARAS* (Fusion Engineering and Design 221, December 2025), report approximately ±20% differences in selected plasma-derived quantities due to different treatments of edge safety factor. They also flag bootstrap-current calculations outside the anticipated future power-plant regime. The differences affect plasma current, beta components and subsequent confinement calculations. This is not a generic ±20% error bar and does not establish that either code is universally wrong; it is evidence that model formulation and validity domain can matter to design decisions. [Paper / DOI](https://doi.org/10.1016/j.fusengdes.2025.115359). [PROCESS publication listing](https://ukaea.github.io/PROCESS/publications/)

### 2.2 Defensible model choices can move the feasible region

D0FUS 3.2.0 documents design sensitivity to alternative elongation closures, kink-stability criteria, density limits and confinement scalings. Its model-uncertainty synthesis repeats the reference EU-DEMO design map with different defensible model choices and shows materially different plasma-stability and machine-feasibility regions. It notes that combined model-choice impacts may be more pronounced than one-at-a-time changes. [D0FUS model uncertainty documentation](https://d0fus.readthedocs.io/en/latest/theory/model_uncertainties.html).

D0FUS's uncertainty execution mode also supports distributions and discrete model switches, Latin-hypercube sampling, feasibility fractions and Wilson intervals. This is a useful baseline to compare with, not something to relabel as a measured probability of plant success. [D0FUS 3.2.0 release description](https://pypi.org/project/d0fus/3.2.0/).

### 2.3 Robust design already has strong precedents

Coleman et al. define a PROCESS-based EU-DEMO design point robust to epistemic confinement uncertainty. Their quantified confinement margin materially reduces potential net electric power compared with a design without the margin. This is exactly the engineering trade-off the decision atlas should make explicit: robustness is not free, and a design must expose the margin's effect on the delivered service objective. [Nuclear Fusion (2025)](https://doi.org/10.1088/1741-4326/ada6d9).

DOE's June 2026 roadmap calls for interoperable model outputs, uncertainty bounds carried through end-to-end analysis, validation against measurements and held-out operating points/facilities, plus RAMI-driven early design screening. [DOE roadmap](https://www.energy.gov/documents/fusion-science-and-technology-roadmap).

## 3. Preserve different forms of uncertainty as different evidence

The evaluator must not flatten these dimensions into one uncertainty score.

| Dimension | Meaning | Owner / existing work |
|---|---|---|
| Parameter uncertainty | uncertainty in declared inputs; correlation/covariance matters | Symtropy [#4652](https://github.com/Luminous-Dynamics/symtropy/issues/4652), [#5333](https://github.com/Luminous-Dynamics/symtropy/issues/5333), [#5400](https://github.com/Luminous-Dynamics/symtropy/issues/5400) |
| Model-form discrepancy | measured residual envelope conditional on observable, model revision, regime and geometry | Symthaea [#12419](https://github.com/Luminous-Dynamics/symthaea/issues/12419), [PR #12420](https://github.com/Luminous-Dynamics/symthaea/pull/12420), [PR #12509](https://github.com/Luminous-Dynamics/symthaea/pull/12509) |
| Model-set disagreement | finite spread / decision-side differences across matched, source-pinned model variants | Symthaea [#12589](https://github.com/Luminous-Dynamics/symthaea/issues/12589) |
| Applicability | whether a model is valid for this candidate, regime, geometry and requested use | Symthaea's validation-domain and discrepancy contracts; specialist model owners |
| Scenario uncertainty | coupled operating, grid, availability, maintenance and supply scenarios | Spark #34–#38 and Symtropy engineering operations |
| Preference uncertainty | stakeholder weights and declared trade-offs among already feasible candidates | Spark #36; full Pareto frontier retained |

**Non-negotiable semantics**

- A finite model-set spread is not automatically a probabilistic confidence interval.
- Solver names do not prove independent evidence; shared source/calibration/closure dependencies must remain visible.
- A covariance matrix is not a confidence interval without an explicit distribution and coverage method.
- Missing cross-covariance is unknown, not zero.
- Model-form residuals do not silently become diagonal entries in parametric covariance.
- Conformal coverage is conditional on the declared calibration/split assumptions; insufficient finite-sample rank is not repaired by clipping.
- OOD, unknown, nonconverged and missing results remain visible and do not silently disappear from the comparison set.
- Different reactor concepts are design alternatives, not samples from one shared uncertainty distribution.
- Models are only compared when model family, observable, unit, boundary, geometry, time basis and operating regime are genuinely matched.

## 4. A small synthetic known-truth example

The following numbers are invented solely to test decision semantics. They are not reactor predictions, plant requirements or engineering recommendations.

Two candidate designs, A and B, are evaluated under two matched alternative model formulations. The hard gates are synthetic net export ≥ 50 MWe and maintenance clearance ≥ 0.08 m. The objectives are maximize net export and minimize lifecycle cost units.

| Candidate | Model | Net export (MWe) | Clearance (m) | Lifecycle cost units | Gate disposition |
|---|---|---:|---:|---:|---|
| A | M1 | 130 | 0.12 | 6.0 | Pass |
| A | M2 | 90 | 0.07 | Fail clearance |
| B | M1 | 91 | 0.11 | Pass |
| B | M2 | 89 | 0.10 | Pass |

Expected evaluator outputs:
- Nominal single-model selection under M1 prefers A on both objectives.
- The finite model envelope for A crosses the clearance gate; A is not robustly feasible over the registered model set.
- B passes both hard gates under both models.
- B must not be described as universally superior; this synthetic example only establishes robust feasibility over this finite declared set.
- No probability is assigned to M1 versus M2. The arithmetic mean of two model outputs is not an uncertainty distribution.

This is the first model-choice design-cliff control. It can be extended into a synthetic Pareto corpus with known fronts and held-out variants.

## 5. Decision Atlas contract

For every candidate revision, preserve:
1. candidate and architecture-family identity;
2. exact model variant, revision, source lineage and declared applicability;
3. scenario / operating-state identity;
4. exact parametric-UQ receipt and its method/coverage disposition;
5. exact model-form discrepancy and calibration/applicability receipts;
6. finite model-set envelope and common-mode dependence profile;
7. hard-gate results and reasons;
8. objective vector, units and frozen normalization;
9. robust / possible / non-dominated / dominated relations;
10. the next evaluation that could change a consequential decision.

The decision atlas should retain separate sets:
- **robustly feasible**: all required registered scenarios/models that are in-domain pass every hard gate;
- **possibly feasible / indeterminate**: evidence incomplete, model disagreement, or unresolved applicability prevents qualification;
- **infeasible**: a required, in-domain qualified model/scenario violates a hard gate;
- **out-of-domain / unsupported**: a solver result cannot support the requested claim.

Do not conflate "not robustly feasible" with "physically impossible". Distinguish a demonstrated failure from insufficient evidence.

### Dominance semantics

Dominance should be computed only over candidates comparable on the same objective boundary, normalization, model/scenario set and gate policy. Report:
- robust dominance across the complete declared evaluation set;
- possible dominance where uncertainty sets permit but do not establish dominance;
- non-comparability where required evidence/model domains do not match;
- scalar preference only after hard gates and Pareto relations, with declared stakeholder weights.

A hard-gate failure cannot be compensated by lower capital cost or higher nominal plasma gain.

## 6. Required synthetic challenge families

All are synthetic or analytic until explicitly replaced by source-pinned solver/measurement datasets.

1. **Model-choice design cliff:** alternative defensible model formulations move one candidate across a hard constraint.
2. **Decision-irrelevant disagreement:** model spread is large but stays on the same side of every decision threshold.
3. **Common-mode consensus trap:** three solvers agree because they share a source dataset, closure or calibration lineage, but an independent known-answer reference is biased relative to that consensus.
4. **Covariance sign trap:** identical marginal variances and different off-diagonal covariance produce different net-power contrast variance for c = [1, −1, −1].
5. **Model-form tail trap:** rare large measured residuals change the conformal allowance; mean-only shrinkage must fail the control.
6. **Finite-sample calibration cliff:** at 95% discrepancy coverage, n = 18 gives k = ceil((n+1)·0.95) = 19 and therefore no finite split-conformal allowance; n = 19 gives k = 19, the maximum observed residual. Do not clip.
7. **Coverage-composition trap:** parametric 97.5% and discrepancy 97.5% give a conservative union-bound floor of 95%; 95% plus 95% only gives 90%.
8. **OOD trap:** unknown/out-of-domain applicability emits no adjusted robust lower bound.
9. **Pareto rank reversal:** the nominal scalar winner is not the robust feasible choice after maintenance/lifecycle/model constraints.
10. **Preference perturbation:** weights change the scalar selection but cannot change feasibility or the underlying Pareto set.
11. **Scenario correlation:** common-cause grid outage, access loss and auxiliary-load changes cannot be treated as independent without a declared joint model.
12. **Missingness and selection:** failed, infeasible, OOD and indeterminate candidates cannot be omitted from the denominator or frontier report.

Each fixture needs explicit known equations, outputs, expected disposition, fixed seed when stochastic, and an oracle that is independent of the evaluator.

## 7. Baselines and tournament

Use identical candidate pools, initial evidence, allowed evaluation budgets and held-out scenarios.

| Baseline | Purpose |
|---|---|
| Nominal single-model optimizer | baseline used by conventional design-point workflows |
| Point-estimate Pareto optimizer | isolates the value of uncertainty-aware feasibility |
| Mean/model-ensemble ranking | diagnostic only; tests the dangerous practice of averaging incompatible models |
| Interval/set-based robust feasibility and dominance | distribution-free baseline when defensible probabilities are unavailable |
| Worst-case / minimax regret | conservative decision baseline |
| Chance constraints | only when calibrated probabilities and joint dependence assumptions exist |
| Symthaea evidence-aware decision policy | proposed research policy |
| Ablations | remove covariance, discrepancy, applicability, dependence, maintenance and lifecycle evidence one at a time |

Hide unobserved reference truth until the policy has committed to its evaluation. Freeze candidate pools, policies, random seeds, budgets, stopping rules, scoreability rules and calibration/validation/holdout partitions before the tournament.

## 8. Metrics and proposed gates

Report separately:
- false robust-feasible and missed robust-feasible rate;
- false dominance and missed dominance;
- robust Pareto precision/recall against known analytic fronts;
- hypervolume, IGD+, epsilon indicator and decision regret;
- worst-case / distribution-free regret where probabilities are unsupported;
- rank-reversal and threshold-decision stability by model variant, scenario and objective preference;
- solver-hours, failed calls, wall time and cost-to-decision;
- interval/bound coverage only when its assumptions and calibration are valid;
- OOD refusal quality and incomplete-case rate;
- dependence/common-mode counts and unresolved source lineage.

Proposed software gates:
- 100% expected classification for frozen analytic threshold, dominance and coverage fixtures;
- zero false robust-dominance and false robust-feasibility claims on the frozen known-truth corpus;
- zero OOD, nonconverged or missing candidate records silently removed;
- zero covariance/model-discrepancy/model-set-spread conflations;
- 100% of probabilistic claims carry the method, calibration and coverage basis;
- deterministic report identity under input reordering;
- every estimated error rate includes its denominator and confidence interval.

Only after the software controls pass should the programme set a numeric research target for regret or expensive-evaluation reduction. That target must be frozen before the held-out tournament and may not be relaxed after results are seen.

## 9. SOTA strategy: compete on decision correctness and evidence efficiency

Do not attempt to beat PROCESS on all systems physics, SOLPS-ITER on edge physics, or OpenMC/Serpent/MCNP on neutron transport. Use them as specialist authorities and matched baselines.

The differentiating result to pursue is: **at the same specialist-evaluation budget, make fewer false-safe decisions, identify unstable design recommendations earlier, and reach a lower-regret evidence-supported decision sooner.**

Report:
- decision regret and false-safe/missed-feasible rates;
- high-fidelity solver-hours per stable decision;
- number of false consensus cases caught;
- time/cost to trace each design constraint to its originating source/model/receipt;
- robust Pareto recovery and rank reversals;
- correct abstention under missing or out-of-domain evidence;
- independent reproducibility and all failed evaluations.

No SOTA claim without matched budgets, blind held-out cases, strong baselines, ablations, repeated seeds, confidence intervals, and an independent verifier.

### Initial implementation state — 18 November 2026

Five JSON fixtures are now present under `benchmarks/research/decision-stability/`. `src/decision_stability.rs` contains Rust known-answer validators and seventeen unit tests, and the Spark fusion case-package workflow runs the targeted test module when the research fixtures or validator change. The fixtures have been fetched back and parsed as JSON; the conformal rank and union-bound values have been independently recalculated in a lightweight static check. This is not a Rust compile/test pass: the local authoring environment has no Rust toolchain and the exact-head hosted result remains queued. The current hosted workflow state is recorded on [draft PR #40](https://github.com/Luminous-Dynamics/spark-engine/pull/40); queued or absent workflows must never be reported as passes. The evaluator covers five synthetic controls, including an OOD numerical-output refusal; the OOD validator retains the model point estimate but refuses decision eligibility and an adjusted robust lower bound, not the full tournament or real solver adapters.

## 10. Execution and ownership

- Spark #44 owns the portfolio-level benchmark and robust Pareto decision interpretation.
- Spark #36 remains the co-design objective/gate programme; Spark #39 / PR #40 remains the case-package contract.
- Symthaea #12589 owns finite matched-model threshold semantics; #12419 / PR #12420 / PR #12509 own model-form residual accounting; #12348 / PR #12349 own net-export reserve consumption.
- Symtropy #4652 / #5333 / #5400 own covariance propagation and exact source-bound receipts.
- Mycelix #1371 / #5900 / #6300 own verifier dependence, experiment attestation and fusion provenance.
- Symtropy #5506 owns geometry and maintenance reachability.
- Sol-Atlas #332 / #662 provide generic grid and site context, not fusion-specific validation.

Sequence:
1. Freeze the benchmark's model/scenario/candidate identity and comparability rules.
2. Implement independent analytic fixtures and exact expected classifications.
3. Define the deterministic Decision Atlas report using the existing Spark case package as the outer evidence envelope.
4. Consume the existing Symtropy covariance and Symthaea model-discrepancy/disagreement receipts.
5. Run paired baseline tournaments with hidden truth and full cost/failure accounting.
6. Independently verify the reports and publish results including failures.

## 11. Claim ceiling

A software PASS establishes only known-answer decision and evidence-composition semantics on frozen synthetic/analytic cases. It does not validate D0FUS, PROCESS, SARAS, plasma physics, materials lifetime, remote handling, a fusion reactor, or physical plant output. No commercial, licensing, safety or operational claim is in scope.
