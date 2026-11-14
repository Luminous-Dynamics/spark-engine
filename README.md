# Spark Engine

A physics-honest Rust toolkit for investigating **Lattice Confinement Fusion (LCF)** —
the anomalous neutron production NASA Glenn Research Center reported from deuterium-loaded
metal lattices under X-ray triggering (Steinetz et al. 2020, *Phys. Rev. C*).

**This is a research tool, not a power source.** Under standard Gamow physics, D-D fusion
at 300 K gives `<σv>` ≈ 5×10⁻⁹⁰ cm³/s and Q ≈ 10⁻⁵⁷ — no net energy gain is possible. The
code enforces this: `QFactor` computes and flags Q < 1 explicitly, it does not let you
assert otherwise. What's real and unexplained is a **~40-to-50-order-of-magnitude gap**
between that prediction and the ~10³ n/s NASA observed. This crate exists to investigate
that gap rigorously — and, if it's confirmed, to design toward the credible application: a
**compact neutron source**, not an energy device.

## What's here

- **Literature-validated core physics** — full Gamow integration (electron screening,
  phonon-enhanced tunneling, D-D multi-channel branching), Q-factor enforcement, DPA-based
  lattice-lifetime/degradation modeling. Parameters checked against ENDF/B-VIII.0, Miracle
  & Senkov (HEA review), and standard radiation-damage references.
- **Anomaly investigation** — `rate_gap` quantifies the observed/predicted gap and its
  possible explanations; `hypothesis_models` implements four candidate mechanisms (transient
  hot spots, coherent phonon cascade, super-screening, lattice-modified tunneling) plus
  measurement error as the skeptical null.
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
- **Neutron source design** — shielding calculations and compact-generator specs, for the
  scenario where the anomaly turns out to be real and controllable.

See [`docs/TECHNICAL_REPORT.md`](docs/TECHNICAL_REPORT.md) (also as
[PDF](docs/TECHNICAL_REPORT.pdf), regenerate with `docs/render_pdf.sh`) for the full
physics-honest assessment and
[`docs/ANOMALY_INVESTIGATION_ROADMAP.md`](docs/ANOMALY_INVESTIGATION_ROADMAP.md)
for the phased research plan this tool supports.

## Materials-mediated benchmark corpus

Spark includes a provenance- and unit-aware record contract for the 2025–2026
metal-hydride D–D literature. It keeps beam-driven metal-target observations,
X-ray-triggered lattice experiments, electrochemical loading controls, relative
yields, model-relative enhancement ratios, total versus areal detector
rates, total versus volumetric reaction rates, calorimetric power, energy
frames, uncertainty status, model revision identity and source rights distinct. The schema foundation is in
[`src/benchmark.rs`](src/benchmark.rs); its admission rules are described in
[`docs/MATERIAL_HYDRIDE_BENCHMARK_DATA_CONTRACT.md`](docs/MATERIAL_HYDRIDE_BENCHMARK_DATA_CONTRACT.md).

This does not yet ingest pointwise experimental data or validate a reactor.
Synthetic unit-test fixtures validate only the data contract.

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
