# Spark Engine

A physics-honest Rust toolkit for investigating **Lattice Confinement Fusion (LCF)** —
the anomalous neutron production NASA Glenn Research Center reported from deuterium-loaded
metal lattices under X-ray triggering (Steinetz et al. 2020, *Phys. Rev. C*).

**This is a research tool, not a power source.** Spark's bare D-D Maxwellian kernel now has a
unit-corrected, total-two-branch constant-S Gamow approximation benchmarked against the
2019 NRL Plasma Formulary over 1–20 keV (35% acceptance envelope). The 300 K solid-lattice
regime is far outside that reference domain: absolute room-temperature rates from this
approximation are extrapolations, not validated standard reactivities. Do not treat a
beam-target screening potential as a measured Maxwellian plasma correction. `QFactor`
continues to flag Q < 1 at room temperature; its absolute low-temperature value remains
model-dependent.

NASA Glenn reported detector-efficiency-corrected fusion-energy neutron rates scaled to
the sample location of 1.5 ± 0.3 × 10³ and 1.6 ± 0.3 × 10³ neutrons/s for two samples.
These are inferred source rates, not raw detector counts or counts per square centimetre.
The mechanism and reproducibility of lattice-mediated results remain research questions.
Spark's goal is to separate these hypotheses, measurements and model predictions rigorously
before any application or scale-up claim.

## What's here

- **Branch-resolved Bosch–Hale reference physics** — separate D(d,n)³He and D(d,p)T
  energy-dependent cross-sections, analytic Maxwellian reactivities, and an independent
  cross-section integration check within the published fit domains. The older constant-S
  Gamow kernel remains a screening-sensitivity approximation, not the authoritative bare
  reference. Screening/phonon hypotheses remain unvalidated model assumptions. See
  [Bosch–Hale branch benchmark](docs/BOSCH_HALE_DD_BRANCH_BENCHMARK.md) and
  [constant-S unit benchmark](docs/DD_MAXWELLIAN_REACTIVITY_REFERENCE_BENCHMARK.md).
- **Anomaly investigation** — `rate_gap` and `hypothesis_models` provide exploratory comparison
  tools for competing hypotheses and measurement-systematic nulls. A quantitative gap is
  publishable only after observed and predicted rates share the same experiment class,
  geometry, source/detector boundary, energy distribution, units and uncertainty chain.
- **Bayesian experiment design** — `bayesian` maintains a probability distribution over
  those five hypotheses and updates it from experimental outcomes; `optimal_experiment`
  ranks candidate experiments by **expected information gain per dollar** and derives a
  budget-constrained sequence. See
  [`docs/DISCRIMINATING_EXPERIMENT_PROPOSAL.md`](docs/DISCRIMINATING_EXPERIMENT_PROPOSAL.md)
  for the generated result — regenerate it with:

  ```bash
  cargo run --example optimal_experiment_proposal 500000   # budget in USD
  ```

- **Uncertainty quantification** — Monte Carlo propagation, Morris screening, tornado
  diagrams, break-even/feasibility analysis.
- **Neutron-source research** — legacy conceptual modelling only. No construction package,
  operational specification, radiation-safety case or qualified source is provided.

See [`docs/TECHNICAL_REPORT.md`](docs/TECHNICAL_REPORT.md) (also as
[PDF](docs/TECHNICAL_REPORT.pdf), regenerate with `docs/render_pdf.sh`) for the full
physics-honest assessment and
[`docs/ANOMALY_INVESTIGATION_ROADMAP.md`](docs/ANOMALY_INVESTIGATION_ROADMAP.md)
for the phased research plan this tool supports.

## Why this might matter to you

If you're doing LCF/LENR-adjacent research and deciding what to fund or run next: the
discriminating-experiment proposal above is a real, regenerable output — not a canned
report. As of this writing it recommends a **Hydrogen Control experiment (~$10K, 1 month)**
as the highest information-value next step, ahead of several six-figure options. Feed in
your own priors (`HypothesisBelief::new(&[...])`) and budget to get your own ranking.

## Usage

```rust
use spark_engine::physics::{GamowIntegration, QFactor, QFactorParams};
use spark_engine::anomaly::AnomalyAnalysis;

let gamow = GamowIntegration::dd_rate(300.0, 309.0, 2);
let (q, _achievable) = QFactor::compute(&gamow, &QFactorParams::lcf_typical());
assert!(q < 1.0); // physics is honest

let observed = 1e3_f64; // NASA neutron rate (n/s)
let predicted = gamow.to_neutron_rate(4.8e22, 0.01); // D density, volume (cm³)
let gap = AnomalyAnalysis::compute_gap(observed, predicted);
```

```bash
cargo test              # 99 tests
cargo run --bin spark   # CLI entry point
```

## License

AGPL-3.0-or-later. Commercial licensing available — see
[`COMMERCIAL_LICENSE.md`](COMMERCIAL_LICENSE.md).

Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics.
