# Fusion systems benchmark strategy — 2026

**Status:** research and qualification programme; targets below are proposed, not achieved results  
**Scope:** evidence-qualified fusion-plant systems integration, not a claim to outperform specialist plasma or neutronics solvers  
**Model-family boundary:** this programme is separate from the repository's LCF anomaly-investigation model family; see [Issue #41](https://github.com/Luminous-Dynamics/spark-engine/issues/41).

## 1. Strategic thesis

The opportunity is not to build another monolithic fusion digital twin or to claim a better plasma-edge, neutronics or systems code. Established programmes already have specialist solvers, facility data and national-laboratory teams.

The opportunity is an **open, evidence-carrying qualification layer** that connects those specialist tools while preserving their physical boundaries, temporal support, units, provenance, uncertainty, applicability and independent review. It should make a fusion design easier to reject early when a critical constraint is unsupported, easier to compare when evidence is available, and easier to reproduce by an external group.

The proposed product is a small "qualification compiler":

1. Parse a design intent and model-family declaration.
2. Bind model inputs and outputs to named physical boundaries.
3. Check units, time basis, surface role, flow direction and conservation.
4. Execute or ingest the output of specialist solvers.
5. Verify model version, configuration, geometry and input/output receipts.
6. Compare against independently frozen references and holdout cases.
7. Propagate uncertainty and model discrepancy without upgrading evidence.
8. Emit a reason-coded qualification result: qualified, not qualified, rejected, indeterminate or out-of-domain.

A successful schema or replay check must never be presented as proof of physical correctness. A result should be qualified only for the exact model, configuration, geometry, operating envelope and evidence scope reviewed.

## 2. Industry and research benchmark landscape

The correct benchmark is different for each physics or engineering discipline. The table below is a dated snapshot of public programme claims and the baseline Spark should use.

| Programme / tool | Public status or capability | What Spark should compare against | What not to claim |
|---|---|---|---|
| **DOE Fusion Science & Technology Roadmap** | June 9, 2026 roadmap calls for FAIR-oriented data infrastructure, common whole-plant verification/validation and uncertainty quantification (VV/UQ), RAMI metrics and handbook, tritium fuel-cycle targets, integrated blanket validation, remote-maintenance qualification and modular digital twins | Treat these as requirements and benchmark categories for our evidence architecture; prioritise fuel cycle, blanket, exhaust, RAMI, net electricity and cross-code integration | A roadmap is not a validated solver, a funded contract or evidence that our work is implemented |
| **UKAEA PROCESS** | Established systems code spanning physics and engineering, heat transfer, energy conversion, buildings and user-selected optimisation objectives; used by DEMO and STEP programmes | Reproduce published cases and compare matched subsystem interfaces; use PROCESS as the first systems-code algebraic/engineering baseline | Do not claim to replace PROCESS's breadth, physical models or institutional validation |
| **OpenMC Fusion Benchmarks** | Public schema-driven, code-agnostic neutronics benchmark framework using standardized YAML specifications, CAD geometry, automatic meshing and reproducible workflows | Reuse its benchmark case structure and geometry/provenance conventions; contribute adapters and additional independently sourced cases | Do not claim our synthetic case packages are neutronics validation, or fork the OpenMC schema without a demonstrated need |
| **SOLPS-ITER / STEP exhaust workflows** | STEP exhaust design work uses databases of SOLPS-ITER simulations to assess reduced engineering models and operating space | Match boundary definitions and compare reduced models with frozen high-fidelity reference cases; keep target profiles, heat-flux width, radiation and geometry explicit | Do not claim a new divertor solver or a better edge model until there are comparable held-out data |
| **CFS SPARC** | CFS currently states a 2027 campaign objective for Q>1; SPARC is still under construction. CFS announced an AI-enabled SPARC digital-twin programme with ten national-lab, university and industry partners in October 2026 | Benchmark integration provenance, uncertainty propagation, operational evidence and reproducible model exchange as a complementary layer | Q>1 is a plasma gain milestone, not whole-plant net electricity or commercial availability |
| **General Fusion LM26** | General Fusion reported electron temperature above 1 keV in October 2026, measured with UKAEA diagnostics; 10 keV and its D-D Lawson milestone remain future programme goals through 2028 | Track plasma-condition claims separately from component/system engineering maturity; test whether our case schema can represent diagnostic evidence and operating-envelope scope | Do not present 1 keV as a Lawson criterion, net fusion gain or net-electric demonstration |
| **Helion Polaris / Orion** | Polaris is described by Helion as a prototype for direct electricity recovery; Orion is the company's planned grid-connected facility with initial operations targeted for 2028 | Benchmark direct-recovery vs thermal-cycle energy ledgers, recirculating-power boundaries, startup schedules, grid delivery and time-phased finance | A design target or power-purchase agreement is not measured net-electric plant performance |
| **NIF** | LLNL reports an 11th ignition experiment on June 20, 2026 with 7.9 MJ yield and target gain about 3.8 | Keep target gain, driver energy, facility electricity and plant-level net energy as distinct quantities and boundaries | Target gain is not wall-plug gain or grid electricity |
| **ITER** | ITER reported the ninth and final vacuum-vessel sector delivered in October 2026, with all core components on site; major assembly and in-vessel work remain | Benchmark large-component supply-chain, configuration, installation, interface and maintenance evidence where suitable public data exist | Component delivery is not plasma operation or a fusion power demonstration |

### Strategic implication

The best positioning is **complementary infrastructure**, not a direct contest to beat facility-backed physics programmes. DOE's roadmap now explicitly values interoperable, traceable whole-plant modelling, FAIR data, RAMI and integrated demonstrations. CFS's October 2026 Genesis Mission project also shows that AI/digital-twin work is being pursued by well-resourced teams. Our differentiator must therefore be specific and demonstrable: open physical-boundary contracts, independent evidence receipts, fail-closed cross-model composition, reproducible comparisons and explicit uncertainty.

## 3. Paradigm shifts worth pursuing

### 3.1 From a monolithic digital twin to a federation of qualified models

No single solver should own all physics. Treat PROCESS, SOLPS-ITER, OpenMC/Serpent/MCNP, thermal-hydraulics tools, structural solvers, remote-maintenance simulations and finance models as specialist authorities in their domains. The shared layer owns the interface contract, evidence state and comparison protocol.

### 3.2 From scalar thresholds to time- and space-resolved mission profiles

A peak heat flux, mean fluence, blanket TBR or component MTBF is not sufficient by itself. Preserve time histories, surface role, geometry, exposure sequence, material state, maintenance events and operating duty cycle. A steady-state scalar may be used only when its reduction assumptions and limitations are explicit.

### 3.3 From a single optimum to evidence-gated Pareto design

There is no universal best reactor. Design trade-offs span confinement, exhaust, blanket, materials, remote maintenance, availability, net electricity, fuel cycle, capital, schedule and lifecycle cost. Hard qualification gates must not be traded away as soft penalties. Produce a Pareto set with evidence status and uncertainty, not one opaque scalar score.

### 3.4 From TBR to time-dependent fuel sufficiency

TBR greater than one is not enough to establish a self-sustaining fuel cycle. Model extraction, processing throughput, holdup, decay, retention, losses, impurity limits, reserve inventory, startup charge, outage states and delivery timing. The DOE roadmap's near-term TBR > 1.1 target is a programme milestone, not a universal sufficiency proof for every concept.

### 3.5 From average availability to executable maintenance campaigns

Availability is determined by the sequence and contention of work: cooldown, isolation, disconnection, cutting/joining, extraction, cask transfers, hot-cell queues, refurbishment, reinstallation, tests and recommissioning. Use event-driven RAMI and actual resource constraints rather than multiplying independent availability percentages.

### 3.6 From model output to evidence-carrying decisions

Every consequential number should have a source, unit, boundary, time basis, model/configuration/geometry revision, uncertainty, domain and validation status. Unknown remains unknown. A digest proves identity, not truth; a signature proves who attested, not physical correctness.

## 4. Benchmark programme and proposed gates

Targets are programme goals. They are not current Spark results, community standards or guarantees of safety.

| Gate | Benchmark | Minimum acceptance target | Stretch target |
|---|---|---|---|
| **K0 — schema** | Positive and negative JSON Schema 2020-12 fixtures on two independent validators | 100% of frozen fixtures classified correctly | Public conformance suite adopted by at least one external adapter |
| **K1 — semantic contract** | IDs, units, boundary roles, time bases, evidence refs, model-family transfers and holdout splits | Zero unresolved references, duplicate IDs or implicit unit/surface/time coercions | Shared cross-repository contract implemented by multiple independent solvers |
| **K2 — conservation** | Analytic power, energy, mass and tritium ledgers | Relative closure ≤ 1e-10 on well-conditioned deterministic fixtures | Conservation checks run automatically on every ingested benchmark case |
| **K3 — numerical replay** | Frozen analytic and solver fixtures | Deterministic replay from pinned inputs/configuration/toolchain; independent implementation agreement within declared numerical tolerance | Reproducible replay across two independent execution environments and maintainers |
| **K4 — exhaust mapping** | PROCESS-matched reduced engineering cases; then held-out STEP/SOLPS-ITER cases | ≤ 1% difference on matched algebraic fixtures after reconciling conventions; publish bias and tail error on higher-fidelity cases | NRMSE ≤ 10% and 90th-percentile absolute relative error ≤ 20% on a preregistered held-out set, if reference quality supports those targets |
| **K5 — transient component loads** | Thermal diffusion oracles, mesh/time-step convergence, public pulsed-heat-load cases | < 1% analytic thermal-response error; < 2% mesh/time-step observable change on declared convergence sequence | < 10% peak-load error and < 1% energy-integral error against qualified transient references |
| **K6 — neutronics** | OpenMC Fusion Benchmarks and independently sourced TBR, heating, activation, shielding and DPA cases | Reproduce official reference cases within their reported Monte Carlo uncertainty with pinned nuclear data and geometry | Multi-code agreement across multiple fusion-relevant CAD geometries with uncertainty coverage |
| **K7 — fuel cycle** | Analytic compartment, decay and delayed-flow fixtures; published inventory cases | Relative mass/inventory closure ≤ 1e-10 in deterministic fixtures; no implicit external fuel source | Held-out prediction intervals with measured coverage and qualified time-to-starvation estimates |
| **K8 — RAMI** | Exact small-system reliability oracles, maintenance-task mock-ups and resource queues | Match analytic limiting cases; zero false-safe classifications in the frozen qualification corpus, with sample count and one-sided confidence bound | Predict outage-duration distributions and annual availability under correlated failures and constrained hot-cell/cask capacity |
| **K9 — whole-plant electricity** | First-law and exergy ledgers against independent calculations | No net-export result if critical recirculating loads or energy boundaries are missing; publish conservation residual | P10/P50/P90 delivered net MWh with availability, ramping, grid constraints and uncertainty |
| **K10 — decision efficiency** | Compare design screening and experiment selection against random, grid, greedy-EIG, cost-aware VOI and independent optimisation baselines | Same feasibility and evidence gates; publish objective, constraint residual, evaluation count and decision regret | At least 50% fewer high-fidelity evaluations to reach the same preregistered Pareto-quality threshold, without reducing validation coverage |

### Rules for a credible SOTA claim

1. Freeze data, code, configuration, geometry, nuclear data, seeds and train/calibration/validation/holdout splits before tuning.
2. Compare on identical boundaries, units, time bases, reaction mix, operating regime and observables.
3. Include simple baselines, a strong specialist baseline, negative controls, ablations and an independent implementation where the mathematics allow it.
4. Report bias, RMSE/NRMSE, tail error, interval coverage, false-safe rate, sample count and confidence bounds.
5. Publish exclusions, out-of-domain cases, failed runs and the exact model limitations.
6. Never compare a campaign target with an achieved result, a plasma Q with plant net electricity, a point estimate with a confidence interval, or one device's calibration data with another device's holdout.
7. A queued, skipped or missing CI job is not a pass; a green software build is not physical validation.

## 5. Execution order

The programme is deliberately sequenced to create a useful evidence chain rather than accumulate disconnected modules.

1. **Case-package contract and semantic validator** — [Issue #39](https://github.com/Luminous-Dynamics/spark-engine/issues/39), [PR #40](https://github.com/Luminous-Dynamics/spark-engine/pull/40). Current work establishes the manifest, model-family isolation, surface/time semantics, semantic validation and a synthetic ledger conformance fixture. Full JSON Schema runtime validation, RFC 8785 canonicalization, RO-Crate packaging and external interoperability are still pending.
2. **Power-exhaust corpus** — [Issue #33](https://github.com/Luminous-Dynamics/spark-engine/issues/33), [PR #32](https://github.com/Luminous-Dynamics/spark-engine/pull/32). Reproduce matched PROCESS cases first; add STEP/SOLPS-ITER and measurements only with public, licensed inputs and exact boundary conventions.
3. **Neutronics adapter and cases** — [Issue #42](https://github.com/Luminous-Dynamics/spark-engine/issues/42). Reuse OpenMC Fusion Benchmarks conventions; preserve geometry, nuclear data, tally definitions, Monte Carlo uncertainty and solver receipts.
4. **Transient component life** — [Issue #38](https://github.com/Luminous-Dynamics/spark-engine/issues/38). Connect qualified time histories to thermal, structural, fatigue, erosion, irradiation and component-life models.
5. **Dynamic tritium closure** — [Issue #37](https://github.com/Luminous-Dynamics/spark-engine/issues/37). Connect blanket neutronics to extraction, processing, inventory, direct recycle, decay and fuel-supply resilience.
6. **RAMI and remote-maintenance campaigns** — [Issue #34](https://github.com/Luminous-Dynamics/spark-engine/issues/34). Use geometry, tool reach, cask/hot-cell capacity, operations, failure and recommissioning evidence.
7. **Whole-plant energy, exergy and net electricity** — [Issue #35](https://github.com/Luminous-Dynamics/spark-engine/issues/35). Consume only qualified subsystem outputs; reconcile first-law energy and exergy destruction before reporting net grid export.
8. **Robust multiobjective co-design** — [Issue #36](https://github.com/Luminous-Dynamics/spark-engine/issues/36). Search a Pareto frontier only over candidates whose mandatory evidence gates are satisfied; keep indeterminate and infeasible states explicit.

## 6. Luminous project boundaries

- **Spark Engine:** case formats, fusion systems economics, power/exhaust interfaces, design screening and the original LCF anomaly investigation. LCF and magnetic-confinement plant model families remain separated.
- **Symtropy:** geometry, thermal/structural/thermal-hydraulic simulations, remote handling, event-driven operations and physical twin receipts.
- **Symthaea:** hypothesis generation, causal analysis, uncertainty attribution and experiment/measurement value-of-information proposals. It does not self-certify its own conclusions.
- **Mycelix:** provenance, immutable receipts, review evidence, inventory/accounting attestations and authority boundaries.
- **Sol-Atlas:** spatial component, geometry, material, irradiation, energy and maintenance evidence layers.

The programme's differentiation is not that every solver is more accurate than the incumbent. It is that cross-model decisions become reproducible, inspectable and fail-closed, and that an external reviewer can trace each decision from source evidence to the final design envelope.

## 7. Public references

- U.S. DOE, [Fusion Science and Technology Roadmap announcement (June 9, 2026)](https://www.energy.gov/articles/energy-department-releases-finalized-fusion-science-and-technology-roadmap-accelerate) and [full roadmap](https://www.energy.gov/documents/fusion-science-and-technology-roadmap).
- UKAEA, [PROCESS systems-code overview](https://www.ukaea.org/service/process/) and [public documentation](https://ukaea.github.io/PROCESS/).
- OpenMC Fusion Benchmarks, [project introduction](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/intro.html) and [benchmark schema](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/specifications/schema.html).
- STEP collaborators, [divertor design and reduced exhaust models](https://doi.org/10.1088/1741-4326/ad93e7).
- Commonwealth Fusion Systems, [SPARC objective and machine status](https://www.cfs.energy/technology/sparc/) and [October 8, 2026 AI-enabled digital-twin project](https://cfs.energy/news-and-media/commonwealth-fusion-systems-to-lead-ai-enabled-digital-twin-fusion-project-in-second-phase-of-genesis-mission/).
- General Fusion, [Q3 2026 results (November 16, 2026)](https://investors.generalfusion.com/news-releases/news-release-details/general-fusion-reports-third-quarter-2026-results).
- Helion, [Polaris prototype](https://www.helionenergy.com/polaris/) and [Orion programme](https://www.helionenergy.com/orion/).
- Lawrence Livermore National Laboratory, [NIF ignition milestones](https://lmf.llnl.gov/science/achieving-fusion-ignition).
- ITER, [delivery of the ninth and final vacuum-vessel sector (October 5, 2026)](https://www.iter.org/node/20687/remarkable-chapter-draws-close).

---

**Current status:** programme strategy and targets only. The open-source implementation has not yet reproduced PROCESS/SOLPS-ITER engineering cases, qualified materials lifetime, demonstrated tritium self-sufficiency, completed whole-plant net-electricity qualification or validated a commercial reactor design.
