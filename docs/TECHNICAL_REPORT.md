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
> The corrected constant-S bare D-D kernel is now regression checked against the
> 2019 NRL Plasma Formulary at 1–20 keV with a 35% approximation envelope. See
> docs/DD_MAXWELLIAN_REACTIVITY_REFERENCE_BENCHMARK.md. The 300 K lattice/beam
> regime is outside that reference domain and remains an extrapolation.

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
only. It is not an uncertainty band on the physical reaction rate. A
full energy-dependent Bosch–Hale branch implementation is the next accuracy
upgrade.

### 1.3 Room-temperature and lattice regime

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
### 2.3 What Could Explain the Gap?

| Enhancement Source | Factor | Cumulative |
|--------------------|--------|------------|
| Electron screening (309 eV) | 10²⁰⁰ | 10²⁰⁰ |
| Phonon enhancement (2 modes) | 10² | 10²⁰² |
| Hot spot (10× T) | 10¹⁰ | 10²¹² |
| **Unexplained** | **10⁻¹⁷²** | 10⁴⁰ |

The combined known enhancements far exceed the gap, but each is applied at its theoretical maximum. In practice, these enhancements may not combine multiplicatively.

> **Honesty caveat (added 2026-07-07):** this table's numbers could not be reproduced
> from the current `screening_enhancement()` implementation (`physics.rs`) at any
> physically relevant energy for this system — the function returns the trivial
> value 1.0 below 10 eV collision energy by design, and is capped at 10⁶ above
> that (added later, per `clippy::manual_clamp` cleanup 2026-07-07). The 10²⁰⁰
> screening figure appears to be from an earlier, uncapped, illustrative
> back-of-envelope exercise evaluated outside the function's intended domain,
> not a live-computed result. Treat this table as **narrative intuition about
> orders of magnitude, not a verified calculation** — which is consistent with
> the original prose above already flagging the multiplicative-combination
> assumption as questionable. Recomputing it rigorously (correct energy
> convention, no artificial floor/ceiling) is open work, not done here.

---

## 3. Hypothesis Evaluation

### 3.1 Hot Spots (Most Plausible)

**Mechanism:** X-ray absorption deposits energy in ~10 nm radius, creating transient regions with T >> T_bulk for ~0.1 ps.

**Predictions:**
- Rate scales with X-ray intensity
- Weak bulk temperature dependence
- Burst structure following trigger pulses

**Required conditions:**
- Peak temperature: ~3,500 K (12× room temp)
- Cooling time: ~0.1 ps
- Plausibility: **HIGH** - thermally achievable

### 3.2 Phonon Cascade (Possible)

**Mechanism:** Multiple optical phonons coherently add their energy (~56 meV each) to the D-D center-of-mass.

**Predictions:**
- Rate increases with acoustic drive
- Resonances at phonon frequencies
- Strong temperature dependence

**Required conditions:**
- Coherent phonons needed: >50
- Plausibility: **LOW** - maintaining coherence of 50+ modes is difficult

### 3.3 Super-Screening (Unlikely)

**Mechanism:** Electron screening energy exceeds measured values due to collective effects at high loading.

**Predictions:**
- Rate correlates with D/M loading
- Should be observable in accelerator experiments

**Required conditions:**
- Screening needed: ~100,000 eV (vs. measured 309 eV)
- Plausibility: **VERY LOW** - 300× higher than any measurement

### 3.4 Measurement Error (Cannot Rule Out)

**Mechanism:** Signal is not from D-D fusion but from background, cosmic rays, or electronic noise.

**Predictions:**
- Signal persists with H instead of D
- No correlation with trigger

**Required conditions:**
- NASA's H control passed
- Plausibility: **LOW** but independent replication needed

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

2. **The anomaly is extraordinary:** A 40-order-of-magnitude gap requires either new physics, unrecognized enhancements, or systematic error.

3. **Hot spots are the most plausible explanation:** X-ray-induced local heating to ~3,500K could explain the observations without invoking new physics.

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
