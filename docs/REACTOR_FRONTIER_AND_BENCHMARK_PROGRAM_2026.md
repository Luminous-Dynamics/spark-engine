# Reactor Frontier and Benchmark Programme — 2026

**Date:** 2026-11-14  
**Status:** Research programme; computational work can proceed, physical validation not claimed  
**Scope:** Spark Engine / Luminous Dynamics  
**Decision principle:** No reactor-energy claim without a complete, independently auditable energy balance.

## Executive direction

Spark Engine currently describes itself as a physics-honest Rust toolkit for investigating lattice-confinement fusion (LCF) observations. It is not a reactor design package and it does not demonstrate a power-producing reactor. Its strongest existing capabilities are Gamow-rate calculations, explicit Q-factor enforcement, anomaly/hypothesis modelling, uncertainty analysis, and expected-information-gain experiment ranking.

The correct next step is not to jump from an anomalous neutron observation to a reactor blueprint. It is to turn Spark into an **evidence-qualified reactor research workbench** with two separate tracks:

1. **Materials-mediated D–D reaction physics:** reproduce and explain the recent metal-hydride results, test whether the effects are predictive across materials and conditions, and determine whether any useful neutron-source application is justified.
2. **Power-reactor architecture comparison:** compare the complete systems behind magnetic, inertial, pulsed and materials-mediated fusion, alongside advanced fission and non-nuclear plasma process reactors. Select a power-plant architecture only when its physics, energy balance, materials, maintainability and regulatory path support that choice.

The tracks share evidence infrastructure but must not share an assumed performance claim. A large reaction-yield enhancement is not energy gain; plasma gain is not wall-plug gain; and wall-plug gain is not net electricity exported to a grid.

## 1. Literature changes that Spark must incorporate

The current repository's July 2026 report predates important 2025–2026 literature. These publications materially change the experimental landscape.

| Result | What was measured | What it does not establish |
|---|---|---|
| Chen et al., *Nature* (20 Aug 2025), DOI [10.1038/s41586-025-09042-7](https://doi.org/10.1038/s41586-025-09042-7) | Electrochemical deuterium loading increased beam-driven D–D fusion rates in a palladium target by an average of 15(2)%. | It was not a self-sustaining lattice reactor. The paper reports approximately 10⁻⁹ W fusion output against 15 W input. |
| Karahadian et al., *Nature Communications* (18 Jul 2026), DOI [10.1038/s41467-026-74421-1](https://doi.org/10.1038/s41467-026-74421-1) | A finite D–D yield floor below approximately 2 keV in Pd and Ti hydrides; the lowest-energy yield was reported as more than 10¹⁸ above the bare-nucleus expectation. The result uses low-energy ion-beam excitation and electrochemical loading, with proton and neutron diagnostics. | The enhancement is relative to a bare-nucleus model, not a reactor Q-factor. The authors explicitly state that condensed-matter fusion is not presently relevant to energy production. Their heterogeneous screening fit is phenomenological, not a unique microscopic explanation. |
| Steinetz et al. / NASA Glenn LCF programme | NASA reports neutron signatures from irradiated deuterated metals and describes a lattice-mediated reaction environment. | The 2025–2026 ion-beam studies are relevant independent evidence about metal-hydride D–D reactions, but they are not replications of the exact NASA X-ray-triggered configuration. |

### Required interpretation rules

- Record **reaction yield**, **yield enhancement**, **screening-fit parameters**, **neutron/proton product rates**, **thermal power**, and **electrical input** as different quantities.
- Label the 10¹⁸ figure as a model-relative enhancement at the lowest reported ion energy. Never describe it as a 10¹⁸-fold energy gain.
- Do not infer a room-temperature reaction rate from an ion-beam experiment without modelling the projectile distribution and energy deposition.
- Treat an effective screening potential inferred from a heterogeneous target as a fitted descriptor, not automatically as a microscopic electrostatic potential.
- Distinguish exact replication, partial replication, related corroboration, and theoretical consistency in the literature database.
- Do not retain unverified screening-enhancement products or order-of-magnitude tables as computed outputs. Every numeric result must be regenerable from an identified model, dataset and parameter set.

## 2. The actual state of the art: separate scoreboards

Headline records across fusion concepts are not interchangeable.

| Architecture / facility | Publicly reported benchmark | Correct reading |
|---|---|---|
| Inertial confinement — NIF | 8.6 MJ fusion yield from 2.08 MJ delivered to the target; target gain 4.13 (7 Apr 2025). LLNL subsequently reported a further 7.9 MJ ignition shot with target gain about 3.8 (20 Jun 2026). | Exceptional target physics. The denominator is laser energy delivered to the target, not full facility electricity; it is not net grid power. |
| Magnetic confinement — WEST | 1,337 seconds of plasma duration (12 Feb 2025). | A sustained plasma-duration result, not a fusion-energy-gain result. |
| Stellarator — Wendelstein 7-X | Triple product of (1.10 ± 0.15) × 10²⁰ m⁻³ keV s held stable for 1.9 seconds (2026 paper). | A strong core-plasma performance milestone; not net power or a commercial availability demonstration. |
| Materials-mediated D–D | 2025 loading effect and 2026 sub-keV yield plateau described above. | A promising condensed-matter nuclear-physics research area; not yet a power-generation route. |

Reference sources:
- [LLNL: Achieving Fusion Ignition](https://lmf.llnl.gov/science/achieving-fusion-ignition)
- [CEA: WEST 1,337-second plasma](https://www.cea.fr/english/Pages/News/nuclear-fusion-west-beats-the-world-record-for-plasma-duration.aspx)
- [W7-X triple-product paper, Physical Review E (2026)](https://doi.org/10.1103/q59k-8m26)
- [DOE Fusion Science & Technology Roadmap (final, 9 Jun 2026)](https://www.energy.gov/documents/fusion-science-and-technology-roadmap)

### Architecture families to compare

| Family | Potential strength | Critical unresolved system issues |
|---|---|---|
| High-field tokamak | Strong magnetic-confinement performance and a comparatively mature engineering ecosystem | Disruptions, steady-state current drive, divertor heat exhaust, neutron-facing components, tritium fuel cycle and remote maintenance |
| Stellarator | Intrinsic steady-state operation and avoidance of tokamak-style current-driven disruptions | Complex precision coils, construction tolerances, maintainability and reactor-scale integration |
| Laser inertial fusion | Demonstrated ignition and high target gain | Wall-plug laser efficiency, repetition rate, target manufacture/injection, chamber clearing, first-wall lifetime and heat extraction |
| Pulsed magnetic / magneto-inertial / FRC / Z-pinch | Potentially compact machines and pulsed energy recovery | Repeatability, electrode/liner or plasma-facing lifetime, pulse-power efficiency, recovery and duty-cycle economics |
| Materials-mediated D–D / LCF | High fuel density in solids; emerging evidence that lattice and defect environments affect low-energy reaction yields | Independent replication of the exact LCF trigger regime, microscopic mechanism, absolute rates, input-power balance, material lifetime and scale-up |
| Advanced fission | Established route to dispatchable nuclear heat and electricity; multiple advanced coolant and fuel concepts | Licensing, fuel supply, construction cost/schedule, waste, supply chain and operating history of novel designs |
| Industrial plasma process reactors | Rapid electrical control and high process temperatures; nearer-term process-heat and chemical applications | Electrode erosion, steep thermal gradients, residence-time control, quenching, process yield and electricity cost |

The 2026 DOE roadmap is especially useful as a gap checklist: it prioritizes structural materials, plasma-facing components, confinement, fuel-cycle systems, blankets, plant engineering, whole-system verification/validation, and maintainability. Its tritium-breeding targets apply to D–T blanket concepts, not to LCF or D–D devices.

## 3. Paradigm shifts

### 3.1 From machine-first to evidence-first architecture selection

Do not decide that the answer is a bigger tokamak, a lattice device, or a pulsed machine before the requirements are defined. Maintain a portfolio of falsifiable concepts and rank them against identical mission requirements: net electric output, process heat, neutron flux, size, availability, fuel logistics, waste, safety case, cost and deployment time.

### 3.2 From bulk material to programmable reaction environment

For the Spark-specific research line, treat the host as an active part of the reaction environment, not as an inert container. Build data models linking phase, composition, loading, defects, strain, grain boundaries, surface condition, diffusion, irradiation history and measured reaction products. This is a research hypothesis programme—not evidence that lattice structure can remove the Coulomb barrier or deliver net energy.

### 3.3 From one fitted parameter to multi-scale causal models

Build a model hierarchy that can be falsified at each layer:

1. Evaluated nuclear cross-sections and branching data.
2. Ion energy distribution, stopping and implantation depth.
3. Hydrogen/deuterium diffusion, loading and desorption.
4. Defect populations, strain and host electronic structure.
5. Heterogeneous screening or other candidate reaction-environment models.
6. Detector response, background, live-time and uncertainty.
7. Complete electrical and thermal energy accounting.

A model must predict held-out observations before it is allowed to guide design. A flexible fit that explains the same points used to fit it is not evidence of mechanism.

### 3.4 From experiment queues to value-of-information research

Spark already ranks candidate experiments by expected information gain. Strengthen this into preregistered discrimination: specify which outcomes support or falsify each hypothesis, freeze priors and cost assumptions, calculate sensitivity to those assumptions, and reserve blinded hold-out data. An EIG ranking is conditional on its prior and outcome model; it is not objectively optimal merely because the code produces a rank.

### 3.5 From plasma Q to useful energy service

Make the full plant the unit of optimization. Track at least:

- Q_target: fusion energy / driver energy delivered to target.
- Q_plasma: fusion power / externally supplied plasma-heating power.
- Driver wall-plug efficiency and recirculating electrical load.
- Gross thermal power and gross electrical output.
- Auxiliary loads, pumps, cooling, cryogenics, fuel processing and maintenance.
- Net electricity exported, process heat delivered, availability and levelized cost.

Never compare values with different system boundaries.

### 3.6 From a design document to an evidence-bearing digital twin

Every design object should carry units, provenance, model version, uncertainty, calibration state, validation domain and a claim status: **measured**, **externally reported**, **modelled**, **extrapolated**, or **unknown**. The twin should preserve residuals and failed predictions instead of overwriting them when the model changes.

## 4. Benchmark programme

Targets below are proposed internal research gates, not claims about present Spark performance or universal industry standards.

### Benchmark A — Literature reproduction and data integrity

**Baseline:** Reproduce published aggregate curves and tabulated rates from the 2025 and 2026 metal-hydride papers, including their stated uncertainty and normalization conventions.

**Targets:**
- Every datum carries source, figure/table, material, isotope, reaction-energy definition, measurement channel, unit and uncertainty.
- Missing values remain missing; no invented digitization or silently imputed uncertainty.
- Reproduce each benchmark curve within its published uncertainty where raw data permit; explicitly flag digitized-only data.
- Keep beam-driven, X-ray-triggered and electrochemical-only conditions as distinct experiment classes.

### Benchmark B — Predictive model discrimination

**Baselines:**
1. Bare-nucleus reference.
2. Conventional homogeneous screened-tunnelling / ALR model.
3. A heterogeneous screening model matching the paper's phenomenological class.
4. Spark's proposed defect- and depth-aware model.

**Targets:**
- Train on one energy/material subset and predict held-out energy bands and a held-out host material.
- Aim for at least 25% lower held-out log-yield RMSE than the strongest simpler baseline, without degrading uncertainty calibration.
- Nominal 90% predictive intervals should cover 85–95% of held-out observations.
- Report model complexity, parameter identifiability and sensitivity to detector/background assumptions.
- Require an independent dataset before describing the new model as better than the state of the art.

### Benchmark C — Replication quality

For a future qualified laboratory collaboration, preregister the sample identity, controls, detector calibration, background collection, exclusion rules, statistical test and raw-data release plan before acquisition.

**Targets:**
- At least three independent facilities for any claim of a general effect.
- At least two host-material families, with isotope and trigger controls.
- Independent product-channel checks where technically available.
- A predeclared significance criterion (suggested minimum: 5σ after background and look-elsewhere treatment).
- Blind analysis and publication of null results as well as positive results.

These are scientific acceptance criteria, not an instruction to construct or operate a neutron-producing apparatus. Experimental work belongs in appropriately licensed specialist facilities.

### Benchmark D — Energy accounting

This is the hard gate between a reaction-physics result and a power claim.

**Targets:**
- Meter all electrical input and relevant thermal input/output across the complete system boundary.
- Independently quantify reaction products and their deposited energy; account for unmeasured channels and uncertainty.
- Publish both the narrow device balance and the whole-system balance, including pumps, cooling, vacuum, drivers, controls and fuel/material processing.
- A power-generation claim requires sustained positive **net electric export** after all parasitic loads, not a target-gain or plasma-gain milestone.
- Before any scale-up proposal, require a closed energy and material balance with uncertainty bounds that do not cross zero net output.

### Benchmark E — Reactor architecture and plant engineering

Use common functional requirements and compare every candidate on:
- net-electric efficiency and net MW (or delivered process heat);
- availability, ramp rate, duty cycle and maintainability;
- fuel-cycle closure and fuel inventory;
- neutron/thermal loading and component lifetime;
- materials qualification and remote maintenance;
- licensing path, safeguards, waste and environmental burden;
- capital intensity, construction duration, supply-chain concentration and cost per delivered MWh.

For D–T concepts, use the DOE roadmap's tritium-breeding, blanket, materials and fuel-cycle metrics as reference requirements. Do not apply D–T tritium-breeding metrics to D–D or lattice concepts.

### Benchmark F — Industrial plasma as a parallel deployment path

Do not make fusion the only route to useful reactor technology. A 2026 *Nature Reviews Clean Technology* review identifies plasma electrification as promising for chemical process decarbonization, while identifying scale-up challenges including electrode erosion, high thermal gradients, reaction-time control and quenching.

Benchmark industrial plasma systems by:
- kWh per kg of on-spec product;
- conversion/selectivity and yield;
- throughput and process uptime;
- electrode/liner lifetime and maintenance interval;
- temperature-time distribution and quench performance;
- lifecycle CO₂ intensity and cost per tonne of product.

This is a separate non-fusion process-reactor line and may be a faster route to industrial value than speculative power generation.

## 5. Execution sequence

### Now — computational, no physical hardware required

1. Add the 2025 and 2026 primary papers to Spark's structured literature corpus with correct classifications.
2. Create a machine-readable benchmark manifest for each dataset, including source provenance and the distinction between absolute and normalized yields.
3. Implement reproducible reference curves for bare-nucleus and homogeneous screened models; verify units, centre-of-mass/lab-frame conversions and stopping-power conventions.
4. Implement held-out energy/material validation and calibrated predictive intervals.
5. Re-audit all numerical claims in the July 2026 technical report, especially screening-enhancement tables and any claimed Q factors. A value that cannot be regenerated should be marked illustrative or removed.
6. Extend the Bayesian experiment ranker with prior-sensitivity and cost-sensitivity analyses. Publish rankings as conditional scenarios, not unconditional prescriptions.
7. Build an architecture scorecard with explicit system boundaries, evidence grades and unknowns.

### Next — collaboration-dependent physical validation

- Seek an academic or national-laboratory collaboration to evaluate the published metal-hydride results and, separately, the exact NASA LCF trigger configuration.
- Prefer shared calibrated datasets, blind reanalysis and cross-facility measurement over an internally claimed prototype.
- No neutron-source product, reactor build or scale-up claim until independent validation, radiation safety, regulatory review and complete energy accounting have passed their gates.

### Later — concept selection

Only rank a candidate as a power-reactor architecture if it has a defensible reaction-rate model, validated component models, an energy-positive system boundary, a plausible fuel cycle, materials lifetime evidence, a maintenance concept and a regulatory pathway. Manufacturing readiness and design-for-inspection must be treated as first-order design constraints, not downstream documentation.

## 6. Immediate Spark engineering backlog

| Increment | Deliverable | Exit condition |
|---|---|---|
| Literature refresh | Structured records for Chen 2025 and Karahadian 2026, clearly marked as beam-driven D–D studies | Sources, dates, DOIs, measurement channels and limits reviewed |
| Benchmark manifests | Versioned, provenance-bearing dataset definitions | Every datum traceable; no silent imputations |
| Model arena | Bare, homogeneous-screening and heterogeneous/depth-aware competitors | Deterministic replay and held-out metrics |
| Uncertainty audit | Unit, frame-conversion, stopping-power and uncertainty tests | Independent reference calculations agree within declared tolerances |
| Experiment-design audit | Prior and cost sensitivity reports for EIG rankings | Ranking stability and failure cases visible |
| Reactor scorecard | Comparative architecture matrix across physics and plant boundaries | No mixed Q metrics; unsupported fields remain unknown |

## 7. Claim discipline and safety boundary

- Spark is a research and engineering decision-support tool, not a reactor, a licensed nuclear facility or a validated reactor digital twin.
- No model output establishes that a physical experiment has occurred.
- No literature result establishes a self-sustaining or net-electric lattice reactor.
- Reactor design, radiation-producing experiments, radioactive-material handling and physical construction require qualified nuclear/plasma/materials engineers, suitable licensed facilities, independent safety review and applicable regulatory authorization.
- All future benchmarks must preserve raw-data hashes, code revision, model configuration, calibration evidence and independent review status.

## 8. Primary references

1. J. A. Steinetz et al., NASA Glenn, [Lattice Confinement Fusion](https://www.nasa.gov/glenn/glenn-expertise-space-exploration/lattice-confinement-fusion/).
2. Y. Chen et al., [Electrochemical loading enhances deuterium fusion rates in a metal target](https://doi.org/10.1038/s41586-025-09042-7), *Nature* (2025).
3. M. E. Karahadian et al., [Enhanced nuclear fusion in the sub-keV energy regime](https://doi.org/10.1038/s41467-026-74421-1), *Nature Communications* (2026).
4. Lawrence Livermore National Laboratory, [Achieving Fusion Ignition](https://lmf.llnl.gov/science/achieving-fusion-ignition), updated with 2026 results.
5. CEA, [WEST beats the world record for plasma duration](https://www.cea.fr/english/Pages/News/nuclear-fusion-west-beats-the-world-record-for-plasma-duration.aspx) (2025).
6. Wendelstein 7-X collaboration, [Tokamak level performance in the optimized stellarator Wendelstein 7-X](https://doi.org/10.1103/q59k-8m26), *Physical Review E* (2026).
7. U.S. Department of Energy, [Fusion Science & Technology Roadmap](https://www.energy.gov/documents/fusion-science-and-technology-roadmap) (final, June 2026).
8. [Scaling up plasma technology for decarbonization of the chemical industry](https://doi.org/10.1038/s44359-026-00201-1), *Nature Reviews Clean Technology* (2026).

---

**Bottom line:** Spark should help us discover which reactor concepts deserve engineering effort, not presuppose that the most interesting anomaly is already a power source. Our competitive advantage should be faster, reproducible design decisions and a higher standard of evidence—then, if the physics earns it, a better reactor.
