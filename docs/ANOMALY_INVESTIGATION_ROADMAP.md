# Spark Engine — Materials-Mediated Fusion Research Roadmap

**Originally proposed:** 2026-02-05  
**Revised:** 2026-11-14  
**Status:** Research programme; not a reactor or neutron-source build specification

## Executive direction

Spark should investigate lattice/material-mediated D–D observations without assuming that an anomaly is established, that a particular mechanism is correct, or that a power-producing reactor is feasible.

The programme has two independent goals:

1. Determine whether the reported material-mediated nuclear signatures are independently reproducible, correctly classified and quantitatively understood.
2. Build a more reliable computational research platform that can compare nuclear reaction models, material hypotheses and reactor architectures without promoting simulation outputs into physical claims.

A neutron source and a fusion power plant are different applications with different energy, materials, safety and economic requirements. No product or manufacturing programme is authorized by this roadmap.

## 1. Corrections to the previous roadmap

The following earlier claims are withdrawn or superseded:

- The quoted 50-order room-temperature gap was calculated using an unqualified low-temperature extrapolation and a numerical prefactor/branch-normalization defect. It must not be used as a benchmark or funding headline.
- The previous 300 K reactivity and Q values are withdrawn pending a separately qualified sub-keV model.
- The old cumulative enhancement table (screening × phonon × hot-spot) was not reproducible and assumed independent multiplicative effects. It is withdrawn.
- Transient hot spots, coherent phonon coupling, super-screening and lattice-modified tunnelling are candidate hypotheses, not ranked explanations.
- Earlier prototype dimensions, materials quantities, shielding concepts, neutron-output targets, market sizes, production costs, revenue forecasts and commercialization dates were illustrative planning assumptions, not engineering requirements or validated projections.
- Claims that a lattice device needs no high voltage, has no radiation hazards, has no tritium-related handling concerns or is exempt from licensing must not be used as safety/regulatory conclusions.

## 2. What the 2020 NASA paper actually reports

Steinetz et al., *Physical Review C* 101, 044610 (2020), DOI [10.1103/PhysRevC.101.044610](https://doi.org/10.1103/PhysRevC.101.044610), report 2.45-MeV neutrons consistent with D(d,n)³He from bremsstrahlung-irradiated ErD₃ and TiD₂ samples. Their paper discusses photodissociation neutrons and neutron-mediated kinetic-energy transfer as part of the proposed mechanism.

The paper reports detector-efficiency-corrected fusion-energy neutron rates scaled to the sample location:
- TS1575: 1.5 ± 0.3 × 10³ neutrons/s using EJ-309.
- TS1576: 1.6 ± 0.3 × 10³ neutrons/s using EJ-309.
- It separately gives a calculated 1.2 × 10³ neutrons/s total for all 16 samples.

These are inferred sample-location source rates—not raw detector counts and not rates per square centimetre. Table V's absolute detection efficiencies (approximately 7 × 10⁻⁶ for EJ-309 and 2 × 10⁻⁶ for stilbene) belong to that paper's detector, shielding, geometry and analysis chain; they are not universal constants.

This programme treats the paper as an externally reported result requiring independent replication and reanalysis. It does not treat it as a measured net-energy device or a validated LCF mechanism.

## 3. Scientific principles and claim discipline

### Separate the relevant physical regimes

Do not merge these experiments into one undifferentiated LCF dataset:
- beam-driven D–D reactions in a metal target;
- X-ray/bremsstrahlung-triggered deuterated lattice studies;
- electrochemical loading controls;
- thermal Maxwellian plasma reactivity;
- other material-mediated nuclear observations.

The 2025 and 2026 beam-driven metal-hydride papers are important materials-physics results but are not replications of the exact 2020 NASA configuration. A model-relative enhancement is not a reaction rate, power output, Q-factor or net energy gain.

### Preserve distinct quantities

Every observation and model output must distinguish:
- incident energy and reference frame;
- absolute reaction yield;
- relative yield and its normalization reference;
- hydride-to-metal control ratio;
- total detector counts/s;
- areal detector counts/(s·cm²);
- total reaction rate (reactions/s);
- volumetric reaction rate (reactions/s/cm³);
- calorimetric power;
- sample-location source rate;
- uncertainty, detector efficiency and calibration.

Unknown uncertainty is not zero. A source-location rate cannot be compared with an areal detector rate without an explicit geometry and response model.

### Keep the system boundaries separate

For future energy applications, distinguish reaction energy, target gain, plasma gain, driver wall-plug efficiency, gross thermal power, gross electrical generation, auxiliary loads and net electricity exported. The 2020 paper's neutron source-rate result is not a plant energy balance.

## 4. Computational baseline and verification

Spark's constant-S bare D–D Maxwellian kernel has received an initial unit and branch correction. It now includes a regression comparison with the 2019 NRL Plasma Formulary over 1–20 keV, with a 35% envelope for the constant-S approximation.

This is a limited verification result:
- it tests bare Maxwellian reactivity with screening and phonon shifts disabled;
- it does not validate 300 K lattice conditions;
- the sub-keV / room-temperature region remains extrapolative;
- it is not a substitute for an energy-dependent Bosch–Hale branch implementation.

### Immediate computational tasks

1. Implement the full energy-dependent D(d,n)³He and D(d,p)T cross-section fits and integrate each branch independently.
2. Cross-check total and branch reactivities against NRL Formulary and Bosch–Hale tables across the fit's valid range.
3. Validate unit, reduced-mass, energy-frame and cross-section conversions against independent reference calculations.
4. Keep the beam-target screening model separate from the Maxwellian reactivity model.
5. Add explicit validity envelopes and refusal behaviour for out-of-domain inputs.
6. Reconcile all published Spark values, including rate-gap and Q tables, from one reproducible source of truth.
7. Preserve failed, non-finite, non-converged and unvalidated results rather than silently dropping them.

Reference benchmark: [DD_MAXWELLIAN_REACTIVITY_REFERENCE_BENCHMARK.md](DD_MAXWELLIAN_REACTIVITY_REFERENCE_BENCHMARK.md).

## 5. Research hypotheses — unranked and falsifiable

### H1 — Measurement or analysis systematics

Potential sources include background, detector response, spectrum unfolding, efficiency correction, timing, selection effects and non-fusion reaction channels.

Research goal: independent, blind reanalysis of raw signals and calibration records, with all exclusions and uncertainty propagation predeclared.

### H2 — Transient local energy deposition

A trigger may create spatially or temporally non-uniform energy distributions. The research requirement is to measure or constrain the actual reactant energy distribution rather than assume a local temperature.

### H3 — Material excitation and phonon coupling

Material excitations may influence energy transfer, screening, diffusion or correlated states. Phonon energies must not be treated as directed centre-of-mass collision energy without a demonstrated coupling model.

### H4 — Dynamic and heterogeneous screening

Static beam-target screening potentials may not describe spatially heterogeneous or time-dependent environments. Measurements must be compared under matched collision energy, material state and diagnostic assumptions.

### H5 — Structure, defects and interfaces

Investigate host phase, grain boundaries, defects, strain, isotope loading, diffusion and surface/interface conditions as explanatory variables. These are material-state covariates, not evidence of a nuclear mechanism by themselves.

### H6 — Model misspecification

The candidate model set may be incomplete. Spark should test an explicit skeptical null and retain an open-world residual category. High expected information gain under one prior is not proof that the registered hypothesis list is complete.

## 6. Evidence and replication programme

Physical experiments belong in qualified specialist facilities under applicable radiation-safety, licensing and regulatory oversight. This roadmap is not an experimental operating procedure.

Before interpreting a future replication:
- preregister the experimental claim, primary observables, null controls, detector calibration and exclusion rules;
- independently review detector response, backgrounds, uncertainty and sample lineage;
- retain raw data and calibration evidence with immutable hashes;
- blind analysis where practical;
- require independent product-channel or diagnostic confirmation when scientifically appropriate;
- publish null results and unresolved discrepancies.

A general effect claim should require independent replication at multiple facilities and host-material families. A single successful reanalysis is not a general mechanism or engineering qualification.

## 7. Reactor and application architecture comparison

Spark must not assume that lattice-mediated D–D is the best energy architecture. The wider portfolio compares:
- tokamak and stellarator magnetic confinement;
- inertial and magneto-inertial fusion;
- pulsed-power concepts;
- advanced fission as an engineering/economic reference;
- industrial plasma process reactors;
- materials-mediated nuclear reaction research.

Compare candidates against declared missions: electricity, process heat, neutron science or another service. Use common system boundaries, material lifetime, availability, maintainability, fuel supply, safety, regulatory path and delivered-energy economics. Do not rank them using incompatible headline metrics.

No reactor is selected for construction by this document.

## 8. Coordinated Luminous responsibilities

- **Spark Engine:** reaction-rate models, LCF literature corpus, nuclear-data comparisons, uncertainty and fusion-specific benchmark records.
- **Symthaea:** source-grounded literature synthesis, competing hypotheses, model-adequacy reasoning, experimental information analysis and scientific representation research.
- **Symtropy:** scenario identity, solver adapters, deterministic replay, material/energy boundary receipts and independent evidence qualification.
- **Specialist facilities and solvers:** qualified numerical calculations and physical measurements in their actual scope.

The cross-repository contract is in [CROSS_REPOSITORY_REACTOR_WORKSTREAM_2026.md](CROSS_REPOSITORY_REACTOR_WORKSTREAM_2026.md).

## 9. Gate sequence

| Gate | Required evidence | What it permits |
|---|---|---|
| G0 — Numerical integrity | Correct units, independent reference comparison, valid domain and exact revision | Publish a computational benchmark result |
| G1 — Data integrity | Source/calibration lineage, raw-data digest, uncertainty and detector-boundary correction | Admit an externally reported observation to the corpus |
| G2 — Independent replication | Preregistered multi-facility replication and independent review | Claim the specified effect is reproducible within its measured scope |
| G3 — Mechanism qualification | Held-out predictions, model discrimination, independent material evidence | Describe a mechanism as supported within its validity domain |
| G4 — Component qualification | Specialist engineering, materials testing, inspectability, lifetime and safety qualification | Consider a component concept within a licensed engineering programme |
| G5 — Plant qualification | Complete energy/material balance, net-output evidence, maintainability, licensing and economic evidence | Evaluate a regulated plant or product proposal |

Passing a software test, digital-twin receipt or simulation gate never substitutes for physical validation, professional engineering approval, radiation protection or regulatory authorization.

## 10. Immediate work queue

1. Finish the full Bosch–Hale branch reactivity implementation and independent known-answer tests.
2. Complete the correction of stale numerical and system-boundary claims across Spark docs.
3. Ingest the 2020 NASA, 2025 Nature and 2026 Nature Communications papers as distinct source records, without fabricating pointwise data.
4. Add the full provenance/unit benchmark contract and hash-backed digitized-data workflow.
5. Build held-out material/energy model comparison and uncertainty calibration.
6. Reuse Symthaea's model-adequacy and experimental-design work.
7. Reuse Symtropy's solver/energy receipt work; do not duplicate plasma or thermal-fluid solvers.
8. Maintain a parallel cross-architecture scorecard so the programme remains valuable even if the LCF hypothesis is not reproduced.

**Bottom line:** The objective is to discover what the evidence supports, correct the physics before optimization, and only advance an engineering concept when its model, measurements, materials and safety case justify it.
