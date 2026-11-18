# Q_econ to divertor power-exhaust boundary contract

**Status:** implemented accounting bridge; physical model adapter and validation programme  
**Revision:** `power-exhaust-boundary-bridge-v0.1`  
**Scope:** connect a Q_econ energy-extraction boundary to separately modelled divertor target loads without conflating their surfaces or claiming independent physical validation.

## 1. Why this boundary matters

Q_econ's `P_f/S` is time-averaged fusion power crossing an energy-extraction control surface divided by that surface's area. Divertor target heat flux is power deposited over a target's wetted area. The denominators and power boundaries are different.

A value in MW/m² is not enough to make the quantities interchangeable. The mapping needs, at minimum, total fusion power on the Q_econ surface, the confined-plasma/separatrix power balance, SOL/divertor radiation, the fraction routed to each target, target wetted geometry, and a peak-to-average heat-flux factor.

This is consistent with established engineering-model practice:
- PROCESS derives divertor wetted area from the SOL width, divertor flux expansion, number of active divertors and target incidence geometry, then divides the residual SOL power by that area. It is an engineering model, not a substitute for a validated edge-plasma solution. [PROCESS divertor model](https://ukaea.github.io/PROCESS/eng-models/divertor/).
- STEP's exhaust scenario work uses a database of SOLPS-ITER simulations to map operational space and validates reduced models against that database; the published paper also identifies uncertainty in SOL heat-flux width, impurity screening and plasma kinetic effects. [STEP exhaust overview, Nuclear Fusion 65 (2025) 016033](https://doi.org/10.1088/1741-4326/ad93e7).
- Experiments on MAST Upgrade show that divertor shape, flux expansion, neutral baffling and detachment control can materially change target loads. The geometric/radiative operating point therefore matters, not merely total fusion power. [Divertor shaping and neutral baffling (2025)](https://www.nature.com/articles/s42005-025-02121-1); [Super-X transient exhaust control (2025)](https://www.nature.com/articles/s41560-025-01824-7).

The present module is a **conservation-ledger adapter**, not a new plasma-edge or divertor solver.

## 2. Implemented accounting contract

`src/power_exhaust_bridge.rs` accepts:
1. An evidence-carrying Q_econ scenario and an explicit extraction-surface identity and area.
2. An effective deposited alpha-heating fraction, auxiliary plasma heating, core radiation and stored-energy change rate.
3. SOL radiation fraction and the fraction of post-radiation SOL power routed to divertor targets.
4. One or more target wetted areas, target power shares and peak-to-average factors.
5. A shared steady-state or finite-window time basis.
6. Separate source, model revision, evidence class and digest receipt for the power ledger and each surface.

The calculation is:

`P_fusion = (P_f/S)_Qecon × A_extraction`

`P_alpha = P_fusion × f_alpha, deposited`

`P_sep = P_alpha + P_aux − P_core-rad − dW/dt`

`P_SOL,post-rad = P_sep × (1 − f_SOL-rad)`

`P_divertor = P_SOL,post-rad × f_divertor`

`P_target,i = P_divertor × share_i`

`q_average,i = P_target,i / A_wetted,i`

`q_peak,i = q_average,i × peak_to_average_i`

All power terms are MW; areas are m²; heat flux is MW/m². Positive `dW/dt` means plasma energy is accumulating and therefore subtracts from separatrix power. Negative `dW/dt` represents release of stored plasma energy and increases the outgoing power.

The implementation rejects:
- mismatch between the declared Q_econ control surface and extraction-surface identity;
- misuse of an extraction surface as a divertor target area (or vice versa);
- missing or duplicate target/surface IDs;
- non-positive/non-finite areas, fractions outside [0,1], or peak factors below one;
- target power shares that do not account for 100% of the routed divertor power;
- a power balance producing negative or non-finite separatrix power;
- missing time-window length for finite-window averages;
- invalid digest syntax or missing source locators for measured, literature-reported or engineering-estimate evidence.

It preserves Q_econ, power-balance and target-surface evidence independently. No evidence class is upgraded. Digest syntax is checked, but canonical serialization and cryptographic verification remain the shared evidence layer's responsibility.

## 3. Deliberate limits

The bridge does **not** calculate or independently validate:
- fusion reaction branching or alpha energy deposition;
- core radiation, separatrix power, SOL radiation or power sharing;
- Eich/heat-flux width, flux expansion, strike-point geometry, baffling or wetted area;
- detachment, neutral transport, impurity screening or transient control response;
- target thermal conduction, coolant performance, stress, erosion, cracking, neutron damage or lifetime.

Those fields must originate from an identified, versioned model or measurement. A scenario assumption can be used for software exploration, but its result remains a scenario assumption. A source URL is provenance metadata, not proof that the cited source supports the numerical value.

The peak-to-average factor is an explicit external input. The bridge must never convert a spatially averaged target heat flux into a peak load by silently assuming a factor of one.

A finite-window average is not a transient-resolved solution. For transients, callers must provide a time-resolved upstream result or a validated peak factor and must retain the time window and temporal resolution. The next API version should represent time histories and energy-integrated loads directly.

## 4. Benchmark plan

No performance claim is made yet. The following are proposed acceptance targets and should be evaluated on frozen, independently sourced cases.

| Gate | Benchmark | Proposed acceptance target |
|---|---|---|
| B0: dimensional/accounting invariants | Power-ledger conservation, target-share accounting, area-role contracts, units and sign conventions | Relative ledger closure ≤ 1e-10 on deterministic fixtures; zero silent boundary substitutions |
| B1: analytic cross-check | Match PROCESS divertor equations using identical declared assumptions and geometry | ≤ 1% relative difference on frozen algebraic cases after reconciling formula conventions |
| B2: geometry | CAD/mesh-derived wetted area against independently reviewed STEP geometry | < 2% area change on mesh refinement; all surface IDs and geometry hashes retained |
| B3: edge/target physics | Compare reduced-model target loads with SOLPS-ITER/validated STEP reduced-model cases | NRMSE ≤ 10% and 90th-percentile absolute relative error ≤ 20% on a preregistered held-out set; report all exclusions |
| B4: transient exhaust | Peak target load and time-integrated deposited energy against transient simulation/diagnostics | < 10% peak-load error and < 1% energy-integral error on cases where the reference temporal resolution is adequate |
| B5: uncertainty calibration | Prediction interval coverage on held-out physical or high-fidelity cases | Nominal 90%/95% interval coverage tested with binomial confidence bounds; no post-hoc interval widening on the holdout |
| B6: safety classification | Cases above/below declared component engineering limits | Zero false-safe classifications in the qualification set; publish sample count and one-sided confidence bound |
| B7: reproducibility | Replay from immutable input, model and geometry receipts | Bitwise-stable deterministic ledger output for fixed toolchain/inputs; independently verifiable source and output digests |

The numeric targets are **proposed programme goals**, not current performance, community-standard thresholds or results already achieved. B3–B6 cannot be scored until datasets, licensing, independent reference calculations and split rules have been agreed.

### Cross-code comparison protocol

1. Freeze a named case, its physical boundaries, time basis, units, target geometry and input digests.
2. Run the exact same case through the bridge and an independent algebraic implementation.
3. For the PROCESS comparison, match the definitions of separatrix power, SOL radiative fraction, active divertors, wetted area and target-angle conventions before comparing values.
4. For higher-fidelity comparisons, map the edge solver's resolved power and target heat-flux profiles into the same named target surfaces; do not compare an upstream parallel heat flux to a target-normal heat flux.
5. Keep calibration and holdout cases disjoint by device, scenario family or operating campaign as appropriate.
6. Report bias, RMSE/NRMSE, tail errors, interval coverage, threshold false-safe rate, sample count and reference-model limitations.
7. Publish failed cases and out-of-domain cases, not only successful fits.

The [OpenMC Fusion Benchmarks](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/intro.html) project is a useful precedent for schema-driven, code-agnostic benchmark definitions, CAD geometry, automated meshing and uncertainty workflows. Its published framework is for neutronics, not divertor heat exhaust; reuse its reproducibility pattern, not its physics assumptions.

## 5. Paradigm shift: make the plant a network of typed physical boundaries

The next-generation architecture should not be a single monolithic fusion "digital twin". It should be a graph of independently qualified transformations:

`fusion source → plasma heating → separatrix → SOL/radiation → divertor targets → thermal response → damage/lifetime → maintenance → availability → net electricity → finance`

Every edge in that graph should carry:
- named input and output physical boundaries;
- dimensional units and a temporal basis;
- model identity, version and configuration digest;
- source and evidence class for every input;
- conservation residuals and uncertainty/discrepancy;
- domain-of-validity declaration;
- holdout benchmark and exact executable/model provenance.

The breakthrough would be an **evidence-carrying plant qualification compiler**: it composes specialist codes without pretending they are one model, catches unit/boundary mismatches before a design is scored, and refuses to promote a design when any critical evidence edge is missing or outside its validated domain.

This is complementary to domain solvers such as PROCESS and SOLPS-ITER. The aim is not to claim a better plasma code; it is to make end-to-end plant decisions auditable and reproducible across codes, vendors and reactor concepts.

## 6. Prioritized next increments

1. Replace point-only bridge inputs with explicit lower/nominal/upper or distribution-bearing inputs, including dependence assumptions and model-discrepancy terms. Use conservative interval propagation only where dependencies are known; do not call independent interval combinations calibrated probability.
2. Add time-history support for ramp-up, steady burn, ELM/disruption transients, detachment transitions and shutdown.
3. Ingest PROCESS output as a first adapter, then a frozen STEP/SOLPS-ITER reference corpus with published licensing and data-split rules.
4. Add CAD geometry receipts and a mesh-convergence record for wetted target surfaces.
5. Couple heat-load histories to material temperature/stress/damage and fluence-limited component life, retaining the full chain of uncertainty.
6. Add the replacement/remote-maintenance queue and hot-cell bottleneck to the RAMI model; calculate calendar availability from event timelines rather than multiplying independent availability factors.
7. Propagate validated component/availability distributions into plant net electricity and time-phased cash flow. Do not treat Q_econ probability as probability of a plant earning revenue.

## 7. Research references

- Whyte et al., “Criteria for the economic viability of fusion power plants,” *Journal of Fusion Energy* 45 (2026), DOI: [10.1007/s10894-026-00577-9](https://doi.org/10.1007/s10894-026-00577-9). This supplies Q_econ and its normalized extraction-surface boundary; it does not supply a divertor mapping.
- PROCESS, [Divertor engineering model](https://ukaea.github.io/PROCESS/eng-models/divertor/). A reproducible reduced engineering-model baseline for SOL width, wetted-area and radiated-power conventions.
- Henderson et al. and STEP collaborators, “An overview of the STEP divertor design and the simple models driving the plasma exhaust scenario,” *Nuclear Fusion* 65 (2025) 016033, DOI: [10.1088/1741-4326/ad93e7](https://doi.org/10.1088/1741-4326/ad93e7). Reduced-model validation against SOLPS-ITER scenario calculations.
- “Divertor shaping with neutral baffling as a solution to the tokamak power exhaust challenge,” *Communications Physics* (2025), [article](https://www.nature.com/articles/s42005-025-02121-1). Experimental evidence that geometry and neutral baffling affect target loads and detachment.
- “Demonstration of Super-X divertor exhaust control for transient heat load management in compact fusion reactors,” *Nature Energy* (2025), [article](https://www.nature.com/articles/s41560-025-01824-7). Experimental transient exhaust control and the need to distinguish average and time-varying load.
- OpenMC Fusion Benchmarks, [schema-driven benchmark framework](https://openmc-fusion-benchmarks.readthedocs.io/en/latest/intro.html). A process/format precedent for benchmark specification, not a divertor reference dataset.
