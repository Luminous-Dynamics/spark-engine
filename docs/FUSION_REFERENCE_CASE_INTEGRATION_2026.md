# Fusion decision-to-engineering reference case

**Status:** proposed integration protocol; implementation and benchmark results are pending  
**Tracking:** [FUSION-E2E-001, Spark Engine #43](https://github.com/Luminous-Dynamics/spark-engine/issues/43)  
**Package owner:** [Spark qualification case package #39 / draft PR #40](https://github.com/Luminous-Dynamics/spark-engine/pull/40)  
**Scope:** composition of existing Symthaea, Spark Engine, Symtropy, Mycelix and Sol-Atlas owners. This is not a new solver or monolithic digital twin.

## 1. Thesis: qualify decisions, not just models

A conventional digital twin often aims to predict a system state. Fusion needs a stronger system: one that can show **why a design decision follows from the evidence, what could reverse it, which engineering operation blocks it, and what measurement would most efficiently resolve the uncertainty**.

The target is a federated research workflow:

`intent → hypotheses → specialist calculations → engineering rehearsal → independent challenge → evidence-qualified decision`

The shared contract should make each interface explicit: model family, physical boundary, quantity, unit, temporal support, geometry, solver/configuration identity, validity envelope, uncertainty, evidence class and terminal disposition.

This is deliberately complementary to specialist codes. PROCESS remains a fusion systems code; SOLPS-ITER/STEP remain specialist exhaust/edge workflows; OpenMC/Serpent/MCNP remain neutron-transport authorities; qualified thermal-hydraulic and structural solvers remain the owners of their domains. The advantage to pursue is not universal solver superiority. It is better **decision quality, reproducibility, evidence traceability and early discovery of false-safe designs across the interfaces**.

## 2. Why this is a current SOTA opportunity

The finalized U.S. DOE Fusion Science & Technology Roadmap (June 9, 2026) calls for modular whole-plant digital-twin infrastructure that connects multiple fidelities and facility data, with quantified error bars, independent holdouts, RAMI, fuel-cycle/blanket integration and remote-maintenance mockups. It specifically identifies maintenance demonstrations that measure time-to-repair, alignment tolerance, error rates and recovery actions. [DOE roadmap](https://www.energy.gov/documents/fusion-science-and-technology-roadmap)

A *Nature Physics* perspective published October 14, 2026 argues that fusion digital twins must combine imperfect models and data through multi-fidelity frameworks while quantifying uncertainty. This supports the research direction, not any claim that a particular Luminous model is already effective. [Fusion digital twins from imperfect models](https://www.nature.com/articles/s41567-026-03485-z)

Public baselines already exist:
- [UKAEA PROCESS](https://www.ukaea.org/service/process/) is a broad systems code spanning physics, engineering, heat transfer, energy conversion and buildings. It is a matched systems-code baseline, not a code we should claim to replace.
- [OpenMC Fusion Benchmarks](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/intro.html) uses schema-driven benchmark specifications, CAD geometry, automatic meshing and reproducible execution for neutronics V&V. Reuse its conventions where compatible rather than inventing a competing neutronics benchmark format. The 2026 paper [OpenMC-Fusion-Benchmarks, a CAD-Based Automated Framework for Fusion Neutronics Validation](https://doi.org/10.1080/15361055.2026.2664345), published online 14 May 2026, describes STEP-based geometries, automated meshing, benchmark specifications and uncertainty analysis; its initial demonstrated case remains a proof-of-concept workflow, not blanket/TBR validation.
- [FNG-ITER streaming validation](https://doi.org/10.1080/15361055.2024.2400762) compares OpenMC with an experimental neutron-shielding benchmark and reports most calculated/measurement discrepancies within combined statistical and experimental uncertainty (less than 10%); it is a shielding validation case, not blanket TBR validation.

## 3. The first end-to-end reference case

### D0FUS 3.2.0 ITER: a negative-net-electric truth control

Reuse the exact pinned execution and output identity recorded in Symthaea [PR #12314](https://github.com/Luminous-Dynamics/symthaea/pull/12314), consumed by [plant service gate #12344](https://github.com/Luminous-Dynamics/symthaea/issues/12344) and [PR #12345](https://github.com/Luminous-Dynamics/symthaea/pull/12345).

The recorded reference outputs are:

| Quantity | Recorded value |
|---|---:|
| Fusion gain Q | 9.959 |
| Thermal power | 549.700 MWth |
| Gross electric power | 192.395 MWe |
| Variable recirculating power | 144.149 MWe |
| Fixed recirculating power | 91.095 MWe |
| Total recirculating power | 235.244 MWe |
| Reconciled net electric power | **−42.849 MWe** |

This is an ITER reference calculation, not an experimental result and not a commercial power-plant design. The values above are rounded for human orientation. The executable case must bind the exact source run, schema, raw/unrounded outputs, artifact digests and output tuple mapping. Do not reconstruct full-precision evidence from the rounded table.

The pinned upstream source is `IRFM/D0FUS v3.2.0`, commit `a682c3511dc189b2a4cdd6f4b8bc545d3cdb8924`; the source/wheel parity work is tracked in [Symthaea PR #12314](https://github.com/Luminous-Dynamics/symthaea/pull/12314). The referenced run is workflow #12, artifact `d0fus-3.2.0-iter-reference-24cf96355086f8bc8b95add8bdb8675c2e124a35`. The frozen output tuple has 165 values and the `GlobalConfig` has 167 fields. The service-gate mapping records `P_e_gross` at tuple index 153, `P_recirc_var` at 160, `P_recirc_fix` at 161 and `P_recirculated` at 162; schema-manifest SHA-256 is `a390b317a336f1ec683e3c2e2950cb7480c7d79bdac38d498258270266b2f774`.

**Precision status:** Symthaea PR #12345 currently uses detailed-report power values rounded to 3–4 decimal places and a 0.001 MW bookkeeping tolerance. Its follow-up D0FUS run is adding full-precision tuple power terms and stricter residual checks. The Spark reference package must use that full-precision artifact before claiming independent numeric reproduction; until then, it is a negative-control specification and a reuse of the source-linked Symthaea service-gate result, not a newly reproduced Spark calculation.

The key negative control is simple:

`P_net = P_gross - P_recirc,var - P_recirc,fixed`

A positive plasma gain and positive gross electric generation must not cause a net-export pass when the reconciled service boundary is negative. The expected terminal disposition is `NetImport`, or `NotEvaluable` if the exact source closure cannot be reproduced. Missing recirculating loads must never default to zero.

### Why this case first

It is already pinned, has a known-answer service-boundary result, and tests an important cross-project interface without pretending to validate a new plasma model. It creates a reproducible failure/control case before introducing a new architecture candidate or higher-fidelity solver adapter.

The next engineering extension can bind one exact geometry/maintenance scenario. Initially this may be synthetic or modelled, but its evidence class must remain exactly that; a deterministic replay is not a physical mockup.

## 4. Existing project responsibilities

### Symthaea — scientific intelligence and decision discovery

Use the existing owners:
- [#7509](https://github.com/Luminous-Dynamics/symthaea/issues/7509), [#7510](https://github.com/Luminous-Dynamics/symthaea/issues/7510), [#7511](https://github.com/Luminous-Dynamics/symthaea/issues/7511): architecture, availability and system-level design search.
- [#7685](https://github.com/Luminous-Dynamics/symthaea/issues/7685): decision-aware multifidelity escalation.
- [#7803](https://github.com/Luminous-Dynamics/symthaea/issues/7803): experiment-selection benchmark by decision impact.
- [#12589](https://github.com/Luminous-Dynamics/symthaea/issues/12589): model disagreement and decision stability.
- [#12344](https://github.com/Luminous-Dynamics/symthaea/issues/12344) / [#12345](https://github.com/Luminous-Dynamics/symthaea/pull/12345): net-electric service gate.
- [#12346](https://github.com/Luminous-Dynamics/symthaea/issues/12346) / [#12347](https://github.com/Luminous-Dynamics/symthaea/pull/12347): event-based lifecycle net-energy/availability ledger.
- [#12565](https://github.com/Luminous-Dynamics/symthaea/issues/12565) / [#12566](https://github.com/Luminous-Dynamics/symthaea/issues/12566): blind measured validation and paired prediction/measurement metrics.

Symthaea proposes candidates and falsifiable hypotheses, builds causal dependency graphs, identifies decision-sensitive assumptions, chooses whether a cheap model is adequate, and proposes measurements. It must preserve competing explanations, failed evaluations, uncertainty and abstentions. It does not authorize hardware operation or independently certify its own outputs.

Architecture-neutral search is a *search-space* property, not permission to apply one model across incompatible physics families. Tokamak, stellarator, IFE and MTF candidates require separate declared model families, specialist solver paths and evidence envelopes. Cross-family comparisons are only made for genuinely comparable observables and explicitly declared decision boundaries.

HDC/CfC is a research candidate for temporal diagnostic interpretation, operating-regime retrieval and anomaly context, not an assumed upgrade over conventional models. Any use must be ablated against simple statistical baselines, Gaussian processes/gradient-boosted trees where appropriate, and established temporal models. Evaluate on blind machine/operating-regime holdouts; report calibration and out-of-domain detection separately from classification accuracy.

### Spark Engine — fusion-specific physics and system accounting

Reuse [power-exhaust PR #32](https://github.com/Luminous-Dynamics/spark-engine/pull/32), [exhaust corpus #33](https://github.com/Luminous-Dynamics/spark-engine/issues/33), [RAMI #34](https://github.com/Luminous-Dynamics/spark-engine/issues/34), [whole-plant electricity #35](https://github.com/Luminous-Dynamics/spark-engine/issues/35), [robust co-design #36](https://github.com/Luminous-Dynamics/spark-engine/issues/36), [fuel-cycle #37](https://github.com/Luminous-Dynamics/spark-engine/issues/37), [component lifetime #38](https://github.com/Luminous-Dynamics/spark-engine/issues/38), [case package #39 / PR #40](https://github.com/Luminous-Dynamics/spark-engine/pull/40), [model-family separation #41](https://github.com/Luminous-Dynamics/spark-engine/issues/41) and [neutronics integration #42](https://github.com/Luminous-Dynamics/spark-engine/issues/42).

Spark owns domain-specific accounting and declared interfaces: fusion systems economics, exhaust power boundaries, heat/electricity ledgers, fuel-cycle and component-service exchanges. It consumes specialist results rather than silently replacing their solvers. The initial LCF anomaly programme and conventional magnetic-confinement plant work remain separate model families; shared types for units, provenance or uncertainty do not transfer physical validity.

### Symtropy — engineering geometry and operations

Reuse [solver-neutral scenario adapter #1685](https://github.com/Luminous-Dynamics/symtropy/issues/1685), [fail-closed solver receipts #1703](https://github.com/Luminous-Dynamics/symtropy/issues/1703), [fusion twin bridge #5851](https://github.com/Luminous-Dynamics/symtropy/issues/5851) and [remote-maintenance geometry #5506](https://github.com/Luminous-Dynamics/symtropy/issues/5506).

Symtropy owns geometry, coordinate frames, rigid-body/kinematic reachability, collision/swept volume, access clearance, payload, deterministic maintenance choreography and scenario replay. The receipt must bind exact candidate revision, CAD asset, frame transforms, robot/tool/payload and solver versions, tolerances, and output digest.

Symtropy's existing rigid-body engine is not, by that fact, a validated MHD, neutron-transport, CFD, thermal-hydraulic or structural finite-element solver. External specialist solvers own those results. Use `EngineeringDeterministic` for qualification-relevant engineering rehearsal; keep interactive visualization separate. Φ-coupled research quantities must not alter SI thermodynamics, plasma physics, heat accounting or material properties.

### Mycelix and Sol-Atlas — independent evidence and site context

Mycelix records evidence provenance, execution identity, independent review and authorization boundaries. Reuse [fusion evidence-provenance bundles #6300](https://github.com/Luminous-Dynamics/mycelix/issues/6300) for power-exhaust source chains, [algorithm-experiment attestations #5900](https://github.com/Luminous-Dynamics/mycelix/issues/5900) for solver/benchmark run identity, and [verifier-independence profiles #1371](https://github.com/Luminous-Dynamics/mycelix/issues/1371) for shared fault-domain transparency. Producer output is not its own independent approval. Hashes establish artifact identity; signatures establish who attested; neither establishes scientific truth.

Sol-Atlas contributes spatial context: site/grid infrastructure, component placement, materials/irradiation layers, maintenance routes and external constraints. Reuse its [grid-network qualification benchmark #332](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/332) and [South African multi-resource siting envelope #662](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/662) for generic grid and siting capabilities; neither is a fusion-specific site qualification. It does not replace specialist physical solvers or issue qualification decisions.

## 5. Canonical dataflow

Every handoff should be an immutable, versioned artifact or a reference to one. Use the Spark case-package contract from #39/PR #40 rather than introducing another package schema.

| Step | Producer | Artifact / result | Consumer | Required guard |
|---|---|---|---|---|
| 1. Intent | Symthaea | candidate ID, model-family, typed design variables, hard constraints, applicability request | Spark + specialist solvers | candidate revision and design assumptions frozen |
| 2. Evaluation plan | Symthaea | fidelity choice, baseline comparison, evaluation reason and budget | solver adapters | hide high-fidelity truth until selection is committed |
| 3. System accounting | Spark | typed boundary flows, power/exhaust/finance/fuel-cycle records | Symtropy + Symthaea | units, surface roles, time basis, source identity, conservation |
| 4. Specialist result | PROCESS / SOLPS-ITER / OpenMC / qualified thermal-material tool | raw outputs, capability profile, convergence and solver receipt | Spark + Symtropy | unsupported, failed, non-converged and incomplete remain explicit |
| 5. Engineering rehearsal | Symtropy | access/reachability/clearance/payload certificate, event/resource schedule and replay receipt | Symthaea | simulation-feasible is not mockup-demonstrated |
| 6. Scientific challenge | Symthaea | uncertainty, discrepancy, rank-reversal, regret, decision stability and next-measurement proposal | human reviewer / research owner | no invented probabilities; no evidence-class promotion |
| 7. Independent review | Mycelix | provenance verification, reviewer disposition and authority record | case package | reviewer independent of producer; digest does not equal truth |
| 8. Context | Sol-Atlas | site/grid/spatial evidence references | Spark + Symtropy | coordinate frame, source date and spatial resolution declared |
| 9. Case decision | case-package assembler | per-gate result and Pareto candidate set | research team | reason-coded states; missing/unknown never passes |

### Evidence states must be non-interchangeable

At minimum, distinguish `Hypothesis`, `Synthetic`, `Modelled`, `CodeCompared`, `Measured`, `MockupDemonstrated`, `IndependentlyVerified` and `PhysicallyQualified`. These are not a simple automatic ladder. For example, an independent check of a synthetic fixture remains synthetic. Evidence may only be reclassified through an explicit, scoped review that names the new supporting evidence.

## 6. Benchmark suite and proposed targets

Targets below are research goals, not existing results, community standards or regulatory acceptance limits.

| Benchmark | Strong baselines / negative controls | Metrics and proposed target |
|---|---|---|
| Case integrity | independent schema validators; stale/mutated digest; missing raw artifact; stale tool/config revision | 100% attempted runs receive a disposition; zero stale/mutated receipts accepted |
| Boundary and conservation | analytic power/energy/mass ledgers; swapped surface, unit, sign, direction and time-base controls | zero silent coercions; exact expected classification for every frozen hostile fixture; relative residual reported |
| Net-electric service | D0FUS ITER reference; Q/gross-electric false positives; missing recirculating loads | reproduce the source-linked net-import result (rounded reference −42.849 MWe); no missing load defaults to zero; no negative-net case is net-export-qualified |
| Multifidelity decision policy | random, space-filling, low-only, fixed-ratio, uncertainty-only, expected improvement and decision-aware regret reduction | aim for ≥25% fewer high-cost solver-hours than fixed-ratio escalation at non-inferior held-out decision quality, with no false-safe increase, following the proposed target in Symthaea #7685 |
| Experiment selection | entropy/EIG, variance reduction, expected improvement, scenario-impact and calibrated EVSI | report decision-regret reduction, cost, decision-change, duration and feasibility separately; EVSI only with calibrated probabilities and conditional utilities |
| Plasma/model credibility | solver convergence controls; independent solver where equations match; blind operating-point/machine holdouts | report bias, RMSE/tail error, interval coverage, calibration, domain coverage and false-safe by regime; do not pool incomparable regimes |
| Neutronics | OpenMC Fusion Benchmarks; FNG-ITER streaming shielding reference; later matched blanket cases | reproduce the selected benchmark within its stated combined uncertainty; separate Monte Carlo, nuclear-data, CAD/mesh and model-form contributions |
| Geometry and maintenance | analytic path/clearance/kinematics cases; worst-case tolerance sweeps; later physical mockups | zero false-safe outcomes on frozen geometry controls; report false-block rate, minimum clearance, payload margin, solver/refinement stability and execution cost |
| Lifecycle service | analytic event ledgers; overlapping outages, derating, standby imports, hot-cell/cask queues and common-cause failures | complete time partition, no double-counted downtime, imports and derated power included, rank reversal from steady-state to lifetime service reported |
| Robust architecture choice | Pareto baselines, fixed search budgets, held-out candidate sets, model-disagreement perturbations | compare hypervolume/regret/feasibility and decision stability; never collapse hard constraints into a compensating scalar |
| Reproducibility | independent package parser/checker and a second execution environment | byte-stable deterministic fixtures or explicitly classified nondeterminism; 100% source/model/config/artifact linkage for scoreable cases |

For zero observed false-safe events, publish the denominator and a one-sided confidence bound. Zero failures in a small synthetic corpus is not evidence of a negligible real-world risk. Freeze data, code, geometry, model revisions, costs, seeds, train/calibration/validation/holdout splits, scoreability rules and stopping criteria before tuning. Preserve failed and abstained evaluations.

## 7. SOTA strategy: compete at the seams

Do not claim to surpass PROCESS in all systems-code physics, SOLPS-ITER in edge plasma, OpenMC/Serpent/MCNP in neutron transport, or ITER/STEP mockup programmes in physical remote handling. Instead, use those as specialist references and establish a measurable advantage in the integrated decision process.

The central leaderboard should be **decision quality per unit of qualified evidence and expensive evaluation cost**:
- false-safe rate and correct abstention;
- decision regret and worst-case regret;
- high-fidelity solver-hours and end-to-end cost-to-decision;
- decision changes captured by an experiment versus information gained but decision unaffected;
- model-rank reversal and robust Pareto-front stability;
- percentage of results whose exact inputs, outputs, solver/configuration, geometry, time basis and evidence are traceable;
- independent replay/verification rate;
- engineering blockers discovered before mockup or hardware spend.

For HDC/CfC, compare conventional baselines and ablate HDC representation, temporal dynamics, physics-informed operator and uncertainty/out-of-domain detection independently. A higher-capacity or more novel architecture is not evidence of superiority. For every SOTA claim, require paired seeds, pre-registered criteria, held-out cases, confidence intervals, ablations, an independent implementation and public reporting of failures.

## 8. Execution sequence

**M0 — contract alignment.** Keep Spark #39/PR #40 as the canonical case-package authority. Cross-link the existing Symthaea/Symtropy issues listed above. Avoid parallel unit, provenance, UQ or optimization frameworks.

**M1 — package the negative control.** Bind the exact D0FUS source run and outputs, model family, schema mapping, service boundaries, gross/recirculating/net ledger and expected `NetImport` disposition. Check exact artifact digests and unrounded data.

**M2 — attach one engineering case.** Bind one geometry/tool/access path to Symtropy #5506 and the Spark exhaust bridge. The first result may be modelled or synthetic. It must retain its evidence class and cannot satisfy a mockup/physical gate.

**M3 — run the decision benchmark.** Use the fixed candidate set and budget to compare policies. Preserve blind high-fidelity outcomes until policy commitment. Report regret, false-safe, abstention, cost and decision stability independently.

**M4 — add a specialist solver adapter.** Start only after the precise upstream input, revision, dependencies, case domain, license, expected outputs and independent reference are all pinned. A successful executable exit is insufficient: convergence and required observable checks are required.

**M5 — independent review.** Verify package semantics and artifact integrity independently, then record a scoped review through Mycelix. No producer self-approval and no automatic evidence promotion.

## 9. Stop conditions and claim ceiling

Stop or return an explicit non-success disposition when:
- source input/output closure or model identity is incomplete;
- model-family or solver capability does not match the candidate;
- a solver fails, times out, does not converge or omits a required output;
- physical boundary, time basis, units, geometry or sign convention is ambiguous;
- required uncertainty, calibration or applicability evidence is absent;
- the geometry/maintenance path is blocked or its tolerance envelope is unresolved;
- a candidate only passes by treating missing loads, losses, outages or dependencies as zero;
- model disagreement crosses a hard decision boundary and remains unresolved.

No live plasma control, ignition, hardware actuation, commissioning, licensing conclusion or autonomous experiment execution is in scope. No software PASS, hash, replay, code-to-code match or optimizer result proves physical validation or commercial readiness.

A successful first release demonstrates **cross-project software and evidence interoperability on a frozen reference case**. It does not demonstrate a better reactor, net-electric fusion power, validated availability, qualified lifetime, physical remote-maintenance readiness or SOTA solver accuracy.
