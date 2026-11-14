# Spark Engine Technical Report: LCF Anomaly Investigation

**Version:** 0.1.0
**Original date:** 2026-02-05
**Revised:** 2026-07-07
**Status:** Physics-Honest Assessment
**Package:** [`spark-engine`](https://crates.io/crates/spark-engine) on crates.io ·
[github.com/Luminous-Dynamics/spark-engine](https://github.com/Luminous-Dynamics/spark-engine)

> **Revision note (2026-11-14):** the former absolute room-temperature reactivity,
> Q and 40-order anomaly-gap figures are withdrawn pending a separately validated
> sub-keV model and matched experiment boundary. A unit-conversion defect in the
> Maxwellian prefactor and a one-branch/total-branch S-factor mismatch were found.
> The legacy constant-S kernel has an NRL 1–20 keV regression check, but it is
> not the bare-nucleus authority. A new branch-resolved Bosch-Hale module now
> implements D(d,n)³He and D(d,p)T cross-sections, analytic Maxwellian reactivities,
> and a numerical cross-section integration cross-check within published domains.
> A second correction fixes identical-reactant rate density from n_D²<σv>/4 to
> n_D²<σv>/2, with branch fractions applied separately. Tests are written but
> not executed here; see docs/BOSCH_HALE_DD_BRANCH_BENCHMARK.md. The 300 K
> lattice/beam regime remains outside these thermonuclear fit domains.

---

## Executive Summary

This report presents a rigorous, physics-honest analysis of Lattice Confinement Fusion (LCF) using the Spark Engine simulation framework. Our key findings:

1. **The present evidence does not establish net energy production.** Room-temperature Q remains below unity under the modelled assumptions, but its absolute numerical value is not a validated physical result.

2. **The prior 40-order anomaly-gap figure is withdrawn.** The bare Maxwellian kernel had a unit-conversion defect; the low-temperature beam/lattice comparison also mixed model and experiment boundaries. A matched, qualified 300 K prediction is still open work.

3. **No lattice mechanism is currently selected as the explanation.** Hot-spot, screening, phonon and measurement/systematic hypotheses remain competing explanations, not established mechanisms.

4. **Any neutron-source application remains conditional on independent replication, calibrated source-rate measurements, material qualification, safety review and regulatory approval.**

5. **The expected-information-gain experiment ranking remains a software-generated research aid**, conditional on its priors, likelihoods, costs and candidate set; it is not a physical validation result.

---

## 1. Standard Physics Baseline

### 1.1 Gamow Integration

The D-D fusion rate is calculated using the full Gamow integral:

```
<σv> = √(8/πμ) · (kT)^(-3/2) · ∫ σ_bare(E) · E · exp(-E/kT) dE
```

Where:
- μ = reduced mass (≈ 1 amu for D-D)
- E_G = Gamow energy = 31.38 keV^0.5 for D-D
- U_e = electron screening energy

### 1.2 Reference benchmark and validity envelope

The previous 300 K table (including the quoted 5.3 × 10⁻⁹⁰ cm³/s and Q value)
is withdrawn. It was generated with an incorrect integration prefactor and
a branch S-factor used as though it represented both D-D branches. Further,
the NRL/Bosch–Hale reference data do not qualify an extrapolation down to room
temperature or a solid-lattice beam-target environment.

The corrected bare, two-branch, constant-S approximation is regression checked
against the NRL Plasma Formulary (2019, p. 45):

| Ion temperature | NRL reference <σv> | Corrected Spark approximation | Difference |
|---:|---:|---:|---:|
| 1 keV | 1.5 × 10⁻²² cm³/s | 1.93 × 10⁻²² cm³/s | +28.7% |
| 2 keV | 5.4 × 10⁻²¹ cm³/s | 5.91 × 10⁻²¹ cm³/s | +9.5% |
| 5 keV | 1.8 × 10⁻¹⁹ cm³/s | 1.65 × 10⁻¹⁹ cm³/s | −8.6% |
| 10 keV | 1.2 × 10⁻¹⁸ cm³/s | 1.01 × 10⁻¹⁸ cm³/s | −15.8% |
| 20 keV | 5.2 × 10⁻¹⁸ cm³/s | 3.89 × 10⁻¹⁸ cm³/s | −25.1% |

The 35% acceptance envelope is for this simplified constant-S regression
only. It is not an uncertainty band on the physical reaction rate. This
constant-S path remains useful for transparent sensitivity experiments but is
not the authoritative bare-nucleus reference.

### 1.3 Branch-resolved Bosch-Hale reference

The new module, src/bosch_hale.rs, implements two separate reference models:
- Energy-dependent D(d,n)³He and D(d,p)T cross-sections using the Bosch-Hale
  polynomial fits, returned in millibarns versus centre-of-mass energy.
- Analytic Maxwellian reactivities for each branch, returned in cm³/s over
  0.2 < T_i < 100 keV.

A separate adaptive-Simpson integration evaluates each branch cross-section
over the Maxwellian relative-energy distribution. Its 1–100 keV conservative
validation interval avoids integrating below the published 0.5 keV
cross-section-fit boundary. The numerical integration and analytic rate fit
agree within approximately 2% in an independent Python check over 1–50 keV;
the Rust regression suite uses a 2.5% envelope and is not yet executed.

The implementation is checked against the MIT Magnetic Fusion Energy
Formulary (2014), §§9.7–9.8, including tabulated cross-sections and branch
rates. The 2019 NRL total-rate table is retained as a separate comparison;
its low-temperature values differ from the Bosch-Hale total and should not be
treated as an exact branch-resolved Bosch-Hale oracle. See
docs/BOSCH_HALE_DD_BRANCH_BENCHMARK.md.

The 0.2–100 keV analytic-fit range is a thermonuclear reference domain, not
a justification for extrapolating to 300 K or a deuterated solid lattice.

### 1.4 Room-temperature and lattice regime

300 K corresponds to approximately 2.6 × 10⁻⁵ keV, far below the 1–20 keV
regression cases above. The GamowIntegration::dd_rate result at 300 K,
especially with screening and phonon shifts enabled, is an extrapolated model
output. It is not a benchmark-validated thermonuclear reactivity, an
experimental rate, or proof of lattice confinement fusion.

Electron-screening data inferred from beam-target collision measurements
cannot be inserted into a thermal Maxwellian model without a clearly stated
kinetic model and validity argument. Likewise, a phenomenological phonon
energy shift is a hypothesis parameter, not a measured increase in the
reactant Maxwellian temperature.

The Q-factor test that room-temperature output is below unity remains a
useful fail-closed invariant. The numerical Q value and the high-temperature
break-even temperature must be recomputed after the full energy-dependent
branch reference and the complete input-power boundary have been qualified.

---

## 2. The NASA Anomaly

### 2.1 Observation Summary

Steinetz et al., *Physical Review C* 101, 044610 (2020), report 2.45-MeV
neutrons consistent with the D(d,n)³He channel in bremsstrahlung-irradiated
ErD₃ and TiD₂ samples. The paper describes deuteron photodissociation and
neutron-mediated kinetic-energy transfer as part of its interpretation.

For two samples, fusion-energy neutron counts corrected for the experimental
detector chain and scaled back to the sample location were reported as:
- TS1575: 1.5 ± 0.3 × 10³ neutrons/s (EJ-309).
- TS1576: 1.6 ± 0.3 × 10³ neutrons/s (EJ-309).

The paper separately reports a calculated rate of 1.2 × 10³ neutrons/s for all
16 samples. The absolute EJ-309 and stilbene detector efficiencies in Table V
are approximately 7 × 10⁻⁶ and 2 × 10⁻⁶ respectively for that specific
geometry and analysis chain. The source-location numbers are not raw detector
counts and are not areal rates. These efficiencies must not be generalized to
other detectors or geometries.

The experimental interpretation is published evidence, not an independent
replication by Spark and not a demonstrated energy-producing device.
### 2.2 Rate comparison status

A quantitative anomaly gap is **not currently publishable from the old defaults**.
The prior predicted value (4.3 × 10⁻³⁸ n/s) used an unqualified low-temperature
extrapolation, a model boundary that was not matched to the sample-location
source rate, and an incorrect numerical prefactor/branch normalization.

The rate comparison must be rebuilt from:
1. a source-qualified experimental record and sample/volume geometry;
2. a declared detector-to-source correction with its uncertainty and calibration;
3. a bare Maxwellian, beam-target or lattice-specific model appropriate to the
   actual reactant energy distribution;
4. separate total, areal, volumetric and detector-count quantities;
5. an uncertainty interval that carries model and geometry uncertainty.

Until those inputs are aligned, do not quote a single gap factor or gap-orders
number.
### 2.3 Candidate mechanisms — no cumulative enhancement factors accepted

No quantitative enhancement factor is assigned to screening, phonon coupling,
transient heating or lattice-modified tunnelling in this report. The previous
10²⁰⁰ screening / 10⁴⁰ cumulative-gap table is withdrawn: it was not reproducible
from the current implementation and combined assumptions without a defensible
joint probability or model.

Each candidate mechanism must be expressed as a separately testable model with:
- an explicit reactant energy distribution and reference frame;
- a measurable observable and competing null explanation;
- a parameter prior and uncertainty;
- a declared range of validity;
- an independent dataset or reference calculation for qualification.

Enhancement factors must not be multiplied together unless a joint physical
model demonstrates that the factors are independent or correctly coupled.

---

## 3. Hypothesis Evaluation

### 3.1 Transient localized energy deposition (unranked hypothesis)

**Mechanism class:** A trigger deposits energy non-uniformly, potentially creating a non-Maxwellian local reactant distribution.

**Discriminating observables:** time-resolved local energy deposition, independently measured reactant energy distributions, trigger-correlated product timing, and controls for detector/electronic transients.

No peak temperature, duration, or fusion-rate enhancement is assumed by this hypothesis.

### 3.2 Phonon/material excitation (unranked hypothesis)

**Mechanism class:** Material excitations may alter local energy transfer, screening or reaction-environment correlations.

**Discriminating observables:** measured phonon spectra, phase/microstructure dependence, isotope controls and held-out predictions across host materials.

A phonon energy is not automatically available as directed centre-of-mass collision energy. Coupling and energy-distribution assumptions must be independently justified.

### 3.3 Dynamic screening and heterogeneous material environments (unranked hypothesis)

**Mechanism class:** Static beam-target screening parameters may not describe time-dependent, spatially heterogeneous electronic/ionic environments.

**Discriminating observables:** screening inferred under matched collision conditions, sample-state lineage, energy-resolved yield, and out-of-material prediction.

Do not extrapolate a measured beam-target screening potential into a thermal Maxwellian without a kinetic model and validity argument.

### 3.4 Measurement and analysis systematics (mandatory null hypothesis)

**Mechanism class:** Background, detector response, unfolding, efficiency correction, timing, selection effects or non-fusion nuclear channels may explain some or all of a reported signature.

**Discriminating observables:** independent detector technologies, blinded reanalysis, calibrated response functions, hydrogen controls, source-location uncertainty and independent replication.

A historical control is relevant evidence but does not remove the need for independent replication.

---

## 4. Literature Survey

### 4.1 Database Summary

| Statistic | Value |
|-----------|-------|
| Total observations | 11 |
| Average credibility | 6.3/10 |
| High-credibility (≥7) | 4 |
| With neutron detection | 2 |
| With excess heat claims | 5 |

### 4.2 Most Credible Observations

1. **Raiola 2004** (9.0/10) - Screening measurement, confirmed by multiple labs
2. **Steinetz 2020** (8.5/10) - NASA neutron observation with controls
3. **Czerski 2004** (8.5/10) - Screening in Ti, confirmed
4. **Huizenga 1993** (7.0/10) - Skeptical review, documents failures

### 4.3 Screening Energy Measurements

| Material | U_e (eV) | Reference |
|----------|----------|-----------|
| Pd | 309 ± 12 | Raiola 2004 |
| Pt | 320 ± 15 | Raiola 2004 |
| Ti | 363 ± 25 | Czerski 2004 |
| Er | ~800 | Steinetz 2020 (estimated) |

These are anomalously large compared to the Debye model prediction (~25 eV) but are well-established by accelerator measurements.

---

## 5. Recommended Experimental Program

### 5.1 Superseding this section: the live, decision-theoretic ranking (added 2026-07-07)

The phase/priority tables below (§5.1–5.3 as originally written) reflect **hand-set
priorities** from the initial design. Since then, two modules make this
decision-theoretic and regenerable rather than fixed:

- `bayesian.rs` — a probability distribution (`HypothesisBelief`) over the five
  candidate explanations (hot spots, phonon cascade, super-screening,
  lattice-modified tunneling, measurement error), with a deliberately skeptical
  default prior (measurement error weighted highest, 0.40).
- `optimal_experiment.rs` — ranks every candidate experiment by **expected
  information gain (EIG) per dollar** against that belief, and derives a
  budget-constrained sequence.

Regenerate the live ranking with:

```bash
cargo run --example optimal_experiment_proposal 500000   # budget in USD
```

As of this writing (see
[`docs/DISCRIMINATING_EXPERIMENT_PROPOSAL.md`](DISCRIMINATING_EXPERIMENT_PROPOSAL.md)),
the ranking **differs from the static table below in its top pick**: the **Hydrogen
Control** experiment — $10K, 1 month — ranks #1 at 9.71 bits of information gain per
$100K, ahead of the NASA Replication and Temperature Mapping experiments the original
static priorities emphasized. This makes physical sense in hindsight: it is the
cheapest experiment that cleanly separates "real fusion" from "measurement artifact,"
the single largest source of belief uncertainty under the skeptical prior. The static
tables below are kept for historical/narrative context but the generated proposal is
now the source of truth for sequencing.

### Phase 1: Validation ($200K, 6 months)

**Goal:** Confirm the anomaly is real and from D-D fusion.

| Experiment | Cost | Priority |
|------------|------|----------|
| NASA Replication | $75K | 100% |
| H Control | $10K | 100% |
| Neutron Spectroscopy | $50K | 95% |

**Go/No-Go:** All three criteria must pass:
- Observe >100 n/s with D
- H control shows <1 n/s
- Neutron energy = 2.45 ± 0.3 MeV

### Phase 2: Mechanism ($300K, 12 months)

**Goal:** Discriminate between hypotheses.

| Experiment | Cost | Priority |
|------------|------|----------|
| Temperature Mapping | $80K | 90% |
| Acoustic Drive | $40K | 80% |
| Host Material Survey | $60K | 85% |

**Go/No-Go:** Identify plausible mechanism.

### Phase 3: Optimization ($200K, 12 months)

**Goal:** Maximize neutron output for applications.

| Experiment | Cost | Priority |
|------------|------|----------|
| Scale-Up | $50K | 70% |
| Lifetime | $30K | 65% |
| Efficiency | $40K | 60% |

**Go/No-Go:** Achieve >10⁶ n/s sustained.

---

## 6. Path Forward: Compact Neutron Source

Even with Q << 1, LCF may be valuable as a neutron source.

### 6.1 Application Requirements

| Application | Rate (n/s) | LCF Viable? |
|-------------|------------|-------------|
| Research | 10⁴ | Yes (if anomaly real) |
| NAA | 10⁶ | Possible with scale-up |
| BNCT | 10⁹ | Challenging |
| Security | 10⁸ | Challenging |

### 6.2 Competitive Comparison

| Source | Rate (n/s) | Cost | Portability |
|--------|------------|------|-------------|
| Cf-252 | 2.3×10⁶/μg | $50K/μg | Portable |
| D-D accelerator | 10⁸ | $200K | Transportable |
| D-T accelerator | 10¹⁰ | $300K | Transportable |
| LCF (if anomaly) | 10⁵/cm³ | ~$20K/cm³ | Portable |

LCF advantage: No high voltage, no tritium, potentially very compact.

---

## 7. Conclusions

1. **Physics is honest:** Standard Gamow gives Q ~ 10⁻⁵⁷ at room temperature. No energy gain is possible.

2. **A quantitative anomaly gap is not currently established:** the former rate comparison used an unqualified low-temperature extrapolation and inconsistent source/detector boundaries. Rebuild it only from matched, traceable data and a qualified model.

3. **No mechanism is presently preferred:** localized energy deposition, phonon/material effects, dynamic screening and measurement-systematic explanations remain competing hypotheses.

4. **Independent replication is critical:** The NASA result must be reproduced with proper controls before investing in scale-up.

5. **Pivot to neutron source:** If the anomaly is real, LCF's value is as a compact neutron generator, not an energy source.

6. **(New, 2026-07-07) The recommended next step, by expected information gain per
   dollar, is the Hydrogen Control experiment** — cheaper and more discriminating
   than the NASA Replication or Temperature Mapping studies emphasized above. See §5.1.

---

## Appendix A: Spark Engine Modules

| Module | Description |
|--------|-------------|
| `physics` | Gamow integration, Q factor, screening |
| `lattice` | DPA tracking, lifetime model |
| `anomaly` | Gap analysis framework |
| `literature` | 11 canonical observations |
| `hypothesis_models` | Hot spot, phonon, screening models |
| `rate_gap` | Quantitative gap calculator |
| `experimental_design` | 3-phase, 9-experiment program |
| `uncertainty` | Monte Carlo propagation |
| `neutron_source` | Compact source design |
| `shielding` | Dose calculations |
| `bayesian` *(added 2026-07-07)* | Probability distribution over the 5 hypotheses, Bayes updates from observations |
| `optimal_experiment` *(added 2026-07-07)* | Expected-information-gain ranking, budget-constrained experiment sequencing |

## Appendix B: Running the Analysis

```bash
# Full analysis report
cargo run --example analysis_runner

# Live discriminating-experiment ranking (added 2026-07-07)
cargo run --example optimal_experiment_proposal 500000

# CLI tool
cargo run --bin spark -- analyze --rate 1000 --material Er
cargo run --bin spark -- hypotheses
cargo run --bin spark -- experiments --phase 1
cargo run --bin spark -- gamow --temp 1000 --screening 309
```

## Appendix C: Test Coverage

- 99 unit tests (up from 42 at original draft), 1 doc test — verified 2026-07-07
- All passing; CI (GitHub Actions) enforces `cargo fmt --check` and
  `cargo clippy -- -D warnings` on every push

## Appendix D: References

- Steinetz, B. M. et al. (2020). "Novel nuclear reactions observed in bremsstrahlung-irradiated deuterated metals." *Physical Review C*, 101(4), 044610.
- Raiola, F. et al. (2004). "Enhanced electron screening in d(d,p)t for deuterated metals." *European Physical Journal A*, 19(2), 283–287.
- Czerski, K. et al. (2004). "Enhancement of the electron screening effect for d + d fusion reactions in metallic environments." *Europhysics Letters*, 68(3), 363.
- Huizenga, J. R. (1993). *Cold Fusion: The Scientific Fiasco of the Century*. University of Rochester Press. (Skeptical review cited in §4.2 as the field's documented failure modes.)
- Brown, D. A. et al. (2018). "ENDF/B-VIII.0: The 8th major release of the nuclear reaction data library." *Nuclear Data Sheets*, 148, 1–142. (Neutron cross-section validation source, per `literature_validation.rs`.)
- Miracle, D. B. & Senkov, O. N. (2017). "A critical review of high entropy alloys and related concepts." *Acta Materialia*, 122, 448–511. (HEA property validation source.)

---

*Report generated by Spark Engine v0.1.0. Original draft 2026-02-05; revised 2026-07-07 alongside the crate's first public release on [crates.io](https://crates.io/crates/spark-engine) and [GitHub](https://github.com/Luminous-Dynamics/spark-engine).*
