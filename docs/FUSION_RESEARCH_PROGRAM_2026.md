# Fusion Research and Engineering Programme — 2026-11

**Status:** Research programme and benchmark specification  
**Scope:** Lattice confinement research, magnetic-confinement fusion, stellarators, field-reversed configurations, inertial fusion, and whole-plant integration  
**Evidence rule:** A model result, synthetic trace, single-laboratory measurement, independently replicated result, and qualified engineering result are different evidence classes.

## 1. Strategic thesis

Luminous Dynamics should not attempt to win fusion by claiming the highest plasma temperature, the largest simulated Q, or a single impressive neural-network score. The programme should make fusion approaches easier to compare, experiments more informative, plant performance more honest, and engineering failures more predictable.

The proposed differentiator is an **evidence-qualified fusion digital thread**:

Physics model → experiment/design decision → instrumented observation → independent validation → plant power balance → lifecycle economics.

Every link carries units, provenance, uncertainty, model version, configuration digest, operating window, and explicit system boundaries.

Fusion approaches remain separate physical hypotheses. Lattice confinement results must not be used as evidence for magnetic-confinement performance, and neither may inherit performance claims from another approach.

## 2. Current evidence and competitive context

### 2.1 Lattice confinement

NASA Glenn's 2025 public LCF material reports nuclear activity in bremsstrahlung-irradiated deuterated metals, including approximately 2.5 MeV neutrons and additional higher-energy neutron observations. NASA describes candidate reactions including D-D fusion and screened Oppenheimer–Phillips stripping, and explicitly states that reaction rates need to increase substantially before appreciable power levels are possible.

Primary sources:
- NASA Glenn, Lattice Confinement Fusion and its 2025 publication set: https://www.nasa.gov/glenn/glenn-expertise-space-exploration/lattice-confinement-fusion/
- Steinetz et al., Novel Nuclear Reactions Observed in Bremsstrahlung-Irradiated Deuterated Metals: https://ntrs.nasa.gov/citations/20250007484

Implication: treat LCF as an anomaly and condensed-matter nuclear-reaction research programme. Do not describe it as an energy-producing reactor. Prioritize independent replication, detector/background controls, full spectra, isotopic controls, synchronized calorimetry and complete trigger-energy accounting before any scale-up claim.

### 2.2 Materials-assisted sub-keV D-D fusion (2026)

A significant new result was published in *Nature Communications* on 18 July 2026:

- Karahadian et al., *Enhanced nuclear fusion in the sub-keV energy regime*, Nature Communications 17, 8845 (2026): https://doi.org/10.1038/s41467-026-74421-1
- Full text: https://pmc.ncbi.nlm.nih.gov/articles/PMC13500673/
- Public data: https://doi.org/10.5281/zenodo.20435774

The study combines a low-energy deuterium ion beam with electrochemical deuterium loading of 250 µm Pd and Ti foils. The center-of-mass beam-energy range is 0.25–6.5 keV. It reports a reproducible relative-yield plateau below approximately 2 keV and an enhancement approaching or exceeding 10^18 relative to a bare-nucleus model at 0.25 keV.

Important measurement boundaries:
- The reported values are **relative thick-target yields**, normalized to the 6.5 keV reference, and enhancement factors relative to a Bosch–Hale bare-nucleus yield model. They are not absolute reactor power or a calorimetric energy-gain measurement.
- Proton spectra are the primary yield diagnostic below 3 keV. Neutron measurements provide an independent check in the overlapping higher-energy regime; do not imply low-energy neutron spectroscopy at every point.
- The study reports more than 350 hours of stable background measurements, no fusion-like signal during electrochemical loading without ion-beam excitation, and greater than 98% gamma rejection in the neutron detector.
- A constant homogeneous screening potential fitted in the 2.5–6.5 keV regime does not describe the lower-energy plateau. The authors use a phenomenological two-environment / energy-dependent effective-screening model; they explicitly state that this is not a unique microscopic explanation.
- At high energy, the fitted effective screening potentials are Pd 0.8 ± 0.2 keV (beam-only) and 1.7 ± 0.2 keV (electrochemically loaded); Ti 0.3 ± 0.2 keV and 0.9 ± 0.2 keV, respectively. These are condition- and model-specific effective parameters, not universal constants for Pd or Ti.
- The paper explicitly says condensed-matter fusion is not presently relevant for energy production.

This is a separate evidence track from NASA's bremsstrahlung-triggered LCF work. The Nature study uses ion-beam excitation and electrochemical loading, with the beam providing the incident energy. It should not be represented as passive, self-sustaining room-temperature fusion or as evidence of net energy gain.

**New Spark Engine priority:** reproduce the published relative-yield curves from the Zenodo data; reconstruct the stopping-weighted thick-target integral; compare constant-screening and heterogeneous-depth models; propagate beam-energy, stopping-power, charge-normalization and counting uncertainties; then test cross-material and loading-regime predictions on held-out data. Preserve the distinction between measured relative yield and the model-derived enhancement ratio.

### 2.3 Magnetic fusion and industry

The DOE's finalized June 2026 Fusion Science and Technology Roadmap organizes common gaps across structural materials, plasma-facing components, confinement, fuel cycle/tritium processing, blankets, and plant engineering/system integration. It explicitly elevates AI, advanced computing, infrastructure and public/private coordination.

Primary source: https://www.energy.gov/articles/energy-department-releases-finalized-fusion-science-and-technology-roadmap-accelerate

Commercial programmes are pursuing materially different routes:
- CFS: high-field HTS tokamak SPARC and the ARC power-plant concept; CFS announced an October 2026 DOE Genesis Mission project for an AI-enabled SPARC digital twin.
- Helion: pulsed FRC approach with magnetic energy recovery and direct electricity conversion as a design objective.
- TAE: beam-driven FRC approach.
- Proxima Fusion: stellarator route based on Wendelstein 7-X experience.
- LLNL/NIF: repeated inertial-fusion ignition; target gain is not facility wall-plug gain or net grid electricity.
- ITER: large international tokamak programme progressing through assembly; it is an experimental programme, not a commercial power plant.

These are competitor reference points, not interchangeable performance claims.

## 3. Paradigm shifts

### P1 — Optimize delivered net electricity, not plasma Q alone

Report at least three separate boundaries:

1. **Primary/device gain:** fusion thermal power divided by energy delivered to the plasma or target. For magnetic confinement this is conventionally related to plasma Q, with the precise heating definition declared.
2. **Whole-plant electrical gain:** gross electrical output divided by all recirculating electrical loads.
3. **Net export:** gross electrical output less all recirculating plant loads, measured at a declared grid boundary.

A machine can achieve a scientifically meaningful plasma gain while still consuming more electricity than it exports. A target-gain result from an inertial-fusion experiment is not a whole-facility electricity result. A direct-conversion design must account for energy recovery, conversion losses, driver loads and repeated-pulse performance.

Spark Engine now provides an initial typed power-boundary model in src/plant_energy_balance.rs. Its engineering result is withheld when the caller says auxiliary loads are incomplete.

### P2 — Co-design the entire plant

Optimize confinement, magnets, blanket, heat extraction, conversion, fuel cycle, maintenance and availability together. Treat replaceability, remote maintenance, component lifetime and planned outages as first-class design variables.

The six DOE roadmap challenge areas form the minimum plant-system decomposition. Each component must expose cost, mass, lifetime, failure modes, test evidence, replaceability and uncertainty.

### P3 — Physics-informed, uncertainty-aware digital twins

Use validated numerical solvers as the physics authority. Use Symthaea HDC/CfC as a representation, forecasting, anomaly detection and decision-support layer. Learn residual corrections only where the training domain and uncertainty are declared. Do not let a learned predictor silently override conservation laws, interlocks or independent protection systems.

Digital twins must support:
- calibration and uncertainty propagation;
- model discrepancy and residual monitoring;
- sensor-quality and missing-data handling;
- out-of-distribution detection;
- synchronized time windows and reproducible replay;
- shadow-mode evaluation before operational use.

### P4 — Active learning for experiments

Extend Spark's expected-information-gain-per-dollar into a robust value-of-information framework that includes:
- cost and schedule distributions, not point estimates only;
- probability of a decisive result;
- expected reduction in decision uncertainty;
- value of a negative result;
- facility access, instrument availability and calibration burden;
- safety, regulatory and reproducibility constraints;
- ability to discriminate competing hypotheses.

An experiment should be funded because its outcome changes a decision, not merely because its nominal information score is high.

### P5 — Qualification as an evidence graph

Every claim should link to raw-data hashes, instrument calibration, experimental protocol, sample provenance, environmental conditions, processing code, model version, uncertainty analysis, reviewer/replicator, and a machine-readable result manifest.

Proposed evidence classes:
- MODEL_OUTPUT
- SYNTHETIC_TEST
- LITERATURE_REPORTED
- MEASURED_SINGLE_SITE
- INDEPENDENTLY_REPLICATED
- ENGINEERING_QUALIFIED

No automated promotion between classes is permitted.

## 4. Symthaea disruption-prediction benchmark

### 4.1 Public baseline

Use MIT PSFC's DisruptionBench as the external baseline:
https://github.com/MIT-PSFC/DisruptionBench

The 2025 paper evaluates C-Mod, DIII-D and EAST across nine tasks and zero-shot, few-shot and many-shot transfer regimes. It reports CCNN intra-machine AUC up to 0.97 on C-Mod and a strongest cross-machine transfer result of AUC 0.83 / F2 0.62 in its reported comparison.

Paper: Spangher et al. (2025), DisruptionBench and Complimentary New Models, Journal of Fusion Energy:
https://doi.org/10.1007/s10894-025-00495-2

These values are reference results, not guaranteed directly comparable targets for a different dataset or split.

### 4.2 Required comparisons

Compare Symthaea HDC/CfC against:
- threshold and rate-of-change baselines;
- random forest;
- the DisruptionBench CCNN;
- the paper's transformer/HDL baselines where code and data permit;
- a conventional temporal model trained on the same features.

Run all published benchmark tasks first, then add machine-transfer tests and sensor-failure tests. Do not use a C-Mod density-limit dataset as a substitute for the public DisruptionBench tasks.

### 4.3 Split and leakage controls

- Split by entire discharge/shot, never by randomly shuffled samples.
- Keep all windows from one discharge in one partition.
- Fit normalizers, encoders, feature selection and thresholds on training/validation only.
- Use training for fit, validation for all model/threshold selection, and one frozen test set for final reporting.
- Include leave-one-machine-out and temporal holdout evaluations.
- Freeze benchmark commit, source-data digest, shot lists, labels, preprocessing, seeds and configuration.
- Report all tasks, not only the strongest one.
- Preserve failed and negative experiments.

The existing C-Mod evaluation implementation in Symthaea requires a leakage audit: threshold selection is performed on the set later reported as test in the V1/V2/V3 paths. Repair this before publishing competitive performance numbers.

### 4.4 Metrics

Report:
- ROC AUC and PR AUC;
- F1 and F2 with declared operating threshold;
- precision, recall and missed-disruption rate;
- false alarms per plasma-hour and per discharge;
- warning-time median, lower quartile and full distribution;
- calibration (Brier score, reliability curves, expected calibration error);
- performance by machine, scenario and operating regime;
- sensor dropout/noise robustness;
- inference latency, compute/memory footprint and data-availability assumptions.

AUC alone is insufficient because it does not describe operational false-alarm burden or usable warning time.

### 4.5 Competitive targets

Phase 1: reproduce published DisruptionBench reference numbers within uncertainty on exactly matched tasks and splits.

Phase 2: match top intra-machine results and improve cross-machine transfer without increasing false alarms or degrading lead time.

Phase 3: demonstrate statistically significant improvements over the strongest matched baseline in at least two independent transfer tasks, with calibrated uncertainty and robust sensor-failure performance.

No numeric superiority claim is allowed until the matched baseline has been reproduced locally.

## 5. Spark LCF research gates

1. **Model audit:** validate units, numerical integration, screening assumptions, reaction branches and sensitivity ranges against published nuclear data and accelerator measurements. Separate measured inputs from extrapolated parameters.
2. **Data integrity:** replace synthetic literature placeholders with source-linked records. Mark synthetic traces as synthetic in type/schema, not only in comments. Store actual raw-data paths/hashes separately.
3. **Independent replication:** require synchronized D/H or appropriate isotope controls, detector calibration, background measurements, neutron spectroscopy, trigger characterization and independent review.
4. **Mechanism discrimination:** test preregistered predictions across independent samples and facilities. Treat measurement/background explanations as live hypotheses.
5. **Application qualification:** only after reproducible neutron output is established, evaluate neutron yield per wall-plug kWh, source lifetime, shielding, maintenance, cost per useful neutron, and regulatory constraints.
6. **Energy claims:** no LCF power-reactor design claim until independent calorimetry and full wall-plug energy balance demonstrate sustained net electrical output.

The NASA 2025 results expand the reaction-observation question. They do not establish an energy source.

## 6. Whole-plant economics

Every proposed plant scenario must specify:
- net export power at the grid boundary;
- availability and duty cycle;
- capital cost and financing/discount assumptions;
- fixed O&M and variable O&M;
- fuel-cycle and blanket costs;
- component replacement and remote-maintenance costs;
- plant life, decommissioning and waste costs;
- grid connection, storage and balancing costs where required.

Report LCOE and sensitivity ranges, not one point estimate. Availability must be derived from an explicit component/maintenance model rather than assumed independently of component lifetimes.

Spark Engine's initial calculate_lcoe model is deliberately first-order. It requires net export, availability, capital, fixed O&M, replacement cost, variable cost, fuel cost, discount rate and lifetime. It is not a bankable project-finance model and callers must include omitted cost classes.

## 7. Cross-project division of responsibility

| Project | Responsibility | Boundary |
|---|---|---|
| Spark Engine | LCF/DD reaction hypotheses, experiment design, energy boundary, scenario economics | Never promote unverified LCF enhancement to power output |
| Symthaea | Plasma diagnostic representations, prediction, disruption and anomaly analysis | Predictive decision support; no independent actuator authority |
| Symtropy | Geometry, deterministic replay, plant/component digital-twin substrate | Specialized MHD, neutronics and thermal solvers remain authoritative |
| Mycelix | Evidence provenance, review records, funding/project governance | Provenance is not scientific validation |
| Sol-Atlas | Site, grid, cooling-water, supply-chain and infrastructure visualization | Data freshness and source provenance must be visible |

## 8. Immediate implementation sequence

1. Add the explicit power-balance/LCOE module to Spark Engine.
2. Replace the Symthaea bridge's temperature-only pseudo-Q with an explicit unavailable/insufficient-input state; never infer Q from temperature alone.
3. Repair C-Mod validation leakage and add deterministic shot-level split manifests.
4. Run Symthaea on DisruptionBench's public tasks and publish frozen baseline results.
5. Reconcile Spark LCF numeric outputs with an independent reference calculation; label unverified mechanisms and synthetic observations clearly.
6. Extend the plant model with uncertainty, availability, component replacement and scenario comparison.
7. Build a common signed evidence manifest and route verified measurements into Mycelix.
8. Connect site and grid scenarios to Sol-Atlas only after the technical plant envelope is credible.

## 9. Sources and reference points

- DOE, Fusion Science and Technology Roadmap, June 2026: https://www.energy.gov/articles/energy-department-releases-finalized-fusion-science-and-technology-roadmap-accelerate
- MIT PSFC, DisruptionBench: https://github.com/MIT-PSFC/DisruptionBench
- Spangher et al. (2025), DOI: https://doi.org/10.1007/s10894-025-00495-2
- NASA Glenn, Lattice Confinement Fusion and 2025 paper set: https://www.nasa.gov/glenn/glenn-expertise-space-exploration/lattice-confinement-fusion/
- LLNL/NIF ignition record: https://lmf.llnl.gov/science/achieving-fusion-ignition
- CFS SPARC: https://cfs.energy/technology/sparc/
- Helion Polaris: https://www.helionenergy.com/polaris
- Proxima Fusion development milestones: https://www.proximafusion.com/about
- Component lifetime and availability study, Fusion Engineering and Design (2025): https://doi.org/10.1016/j.fusengdes.2025.115231
