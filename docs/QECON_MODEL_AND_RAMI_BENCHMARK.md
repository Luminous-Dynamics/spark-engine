# Fusion plant Q-metrics, economics and RAMI qualification

**Status:** research/implementation plan, 17 November 2026  
**Scope:** first-order model qualification, not a power-plant performance claim.

## Why this programme needs multiple metrics

No single fusion "Q" captures commercial readiness. We keep the following quantities separate:

| Quantity | What it answers | What it does not establish |
|---|---|---|
| Plasma/primary gain (Q_physics) | Is the plasma or target producing fusion energy relative to the energy coupled to it? | Whole-plant net electricity, uptime or economics |
| Electrical plant boundary | Is gross electricity sufficient to cover all declared recirculating electrical loads? | Component life, grid value or financeability |
| Q_econ | Do normalized economic gain rates exceed the target, replaceable-surface and fixed cost rates in the published steady-state model? | Bankability, licensing, actual component reliability or achievable power density |
| Surface utilization (U) | What fraction of the idealized operating/replacement cycle is available after the fluence-limited surface replacement interval? | Total plant availability under all component failures and outages |
| RAMI / availability model | How do component failure, repair, inspection, maintenance and recovery affect expected plant uptime? | Commercial profitability by itself |
| Full project-finance model | Are time-resolved cash flows, risk, tax, capital structure and market participation viable? | Scientific truth of the physical models supplied to it |

This separation matters because apparently impressive plasma gain can coexist with negative plant export, short component lifetime, long maintenance outages or unfinanceable capital costs.

## Q_econ implementation

The Rust module `src/qecon.rs` implements the steady-state framework in Whyte et al. (2026), *Criteria for the economic viability of fusion power plants*, Journal of Fusion Energy 45, article 49 ([DOI 10.1007/s10894-026-00577-9](https://doi.org/10.1007/s10894-026-00577-9)).

The ten controlling parameters are:
1. Fusion power density crossing the control surface, P_f/S (MW/m²).
2. Surface energy-fluence limit, X_S (MW·year/m²).
3. Surface replacement interval, tau_rep (years).
4. Whole-system net energy conversion efficiency, eta_E = P_net/P_f.
5. Net selling value of the energy product after variable O&M (USD/MWh).
6. Consumable target cost per target energy yield (USD/MJ).
7. Replaceable-surface and associated component replacement cost (MUSD/m²).
8. Plant construction/delivery plus fixed O&M cost normalized to S (MUSD/m²).
9. Averaged real interest rate (percent).
10. Plant lifetime (years).

The implementation calculates:

```text
U       = 1 / (1 + tau_rep * (P_f/S) / X_S)
C_gain  = 8.76e-3 * POE_net * eta_E * (P_f/S) * U
C_target= 31.5 * (c/Y)_target * (P_f/S) * U
C_S_rep = (MUSD/S)_S * (P_f/S) / X_S * U
C_fixed = (MUSD/S)_FPP * CRF(real_rate, lifetime)
Q_econ  = C_gain / (C_target + C_S_rep + C_fixed)
C_net   = C_gain - C_target - C_S_rep - C_fixed
```

The threshold solver implements the paper's closed-form power-density solution for a requested Q_econ. No implicit default for any of the ten inputs is supplied. Every caller must provide a complete scenario.

The test fixture is an explicit reconstruction of the paper's illustrative case, not a prediction. The paper reports a break-even threshold around 1.9 MW/m², Q_econ around 1.5 at roughly 3.7 MW/m², and an effective LCOE around $67/MWh for its illustrated market assumptions. Those values are regression anchors for the published equations, not universal reactor design limits.

### Boundary rules

- `eta_E` is net power divided by fusion power, not just the blanket's thermal-to-electric efficiency.
- Spark exposes a helper that derives this electrical `eta_E` from a complete `PlantPowerBalance`. An incomplete auxiliary ledger is rejected.
- `U` in this model captures only the idealized surface-fluence / replacement cycle. It is not total plant availability.
- The Q_econ paper's effective LCOE proxy excludes some elements of a full NPV-based valuation. It is kept distinct from Spark's existing first-order LCOE calculator.
- Q_econ >= 1 is a necessary condition in the published steady-state framework, not proof of commercial viability. The paper itself describes the condition as necessary but insufficient.
- The caller-supplied SHA-256 in `QeconScenario` is a provenance receipt. The present module checks digest syntax and carries it through; canonical serialization and digest verification must be provided by the evidence pipeline.

## RAMI and the Q_ENG distinction

A September 2026 paper, Tesini, *An availability-centric Q-metrics framework for assessing fusion power-plant engineering viability*, Fusion Engineering and Design 230, 115874 ([DOI 10.1016/j.fusengdes.2026.115874](https://doi.org/10.1016/j.fusengdes.2026.115874)), proposes an Engineering Burden Index (EBI) and defines its Q_ENG as achievable operational availability divided by EBI.

**Important nomenclature warning:** that paper's Q_ENG is not the conventional electrical engineering gain ratio. It is an availability/engineering-burden construct. The same work distinguishes Q_GLOBAL from Q_PLANT. Spark must not use the name Q_ENG for net electric gain or merge these meanings.

The Q_ENG/EBI paper is a useful research direction, but its published concept values and normalization bands are described as indicative and judgement-based, not experimentally validated or community-standard. Accordingly, Spark will not hard-code those bands as accepted engineering truth. We will implement independently auditable RAMI quantities first: component MTBF, MTTR, scheduled outage duration, parallel maintenance capacity, remote-handling recovery, common-cause failures, hot-cell throughput and uncertainty in outage duration. Any derived EBI-style score will be clearly labelled as a versioned assessment rubric, not a physical constant.

## Benchmark programme

### Gate 1 — equation and unit conformance

- Reproduce the cited Q_econ break-even and Q_econ≈1.5 examples within their published rounding.
- Validate every conversion factor and unit boundary independently.
- Check the closed-form threshold against a numerical root of C_net = 0 and Q_econ(P_f/S) = target.
- Test limiting cases: zero replacement downtime, zero target cost, no net conversion, and no finite solution for an unattainable target.

### Gate 2 — sensitivity and uncertainty

- Reproduce the paper's qualitative sensitivities, not just one point.
- Run seeded Monte Carlo from stated distributions; publish P(Q_econ >= 1), P(Q_econ >= 1.5), P(U >= 0.8), expected C_net and tail risk.
- Include correlations: high power density may worsen surface damage; higher fluence limits may reduce heat-removal performance; remote-maintenance complexity may raise both replacement duration and cost.
- Separate aleatoric variability, epistemic uncertainty and model-form uncertainty. Do not assign narrow distributions to parameters with no evidence.
- Use global sensitivity (e.g. Sobol indices) to rank which measurements or design changes most reduce uncertainty in viability.

The 2026 Q_econ paper itself uses Monte Carlo distributions and emphasizes the importance of uncertainty in the design parameters. The next code increment should therefore extend the deterministic kernel with a reproducible, correlation-aware uncertainty layer rather than inventing point estimates.

### Gate 3 — plant availability and RAMI

Build a separate availability model; never silently substitute surface utilization for plant availability. At minimum, compare:
- surface replacement downtime;
- planned inspection and maintenance;
- component failure and repair distributions;
- remote-handling tool reliability and rescue time;
- common-cause and cascading failure;
- serial versus parallel maintenance;
- hot-cell transport, repair and qualification queueing;
- spares and component refurbishment supply chains.

Report calendar availability, productive utilization, planned/unplanned outage fractions, MTTF, MTTR, repair completion tails, and uncertainty intervals. Only merge these effects into economics through a clearly versioned integration layer.

### Gate 4 — independent physics and engineering V&V

- Use OpenMC-Fusion-Benchmarks as a neutronics V&V reference, not as a Spark-owned solver. Its 2026 framework introduces schema-driven benchmark definitions, STEP CAD geometry, automated meshing and uncertainty workflows ([paper DOI 10.1080/15361055.2026.2664345](https://doi.org/10.1080/15361055.2026.2664345)).
- For plasma disruption prediction, use DisruptionBench's C-Mod, DIII-D and EAST tasks with whole-shot splits and operational trigger logic. Its reported transfer AUC/F2 and intra-machine results are matched-baseline targets, not results Spark has already beaten ([paper DOI 10.1007/s10894-025-00495-2](https://doi.org/10.1007/s10894-025-00495-2)).
- Preserve data licensing, code/model versions, calibration, source hashes, holdout identities, negative results and exact-head test evidence in each evaluation receipt.

## Industry comparison and our credible differentiation

The DOE's June 2026 Fusion Science and Technology Roadmap explicitly calls for an AI-Fusion Digital Convergence Platform integrating HPC codes, physics-constrained digital twins, foundation models, surrogates, whole-facility modelling and real-time control. It also identifies the lack of validated end-to-end plant-level integrated modelling and the need to make RAMI a first-class quantitative design driver ([DOE roadmap](https://www.energy.gov/documents/fusion-science-and-technology-roadmap)).

That is a national-scale programme with access to major facilities, data, compute and public-private partnerships. Spark should not claim to out-scale it. A credible differentiation is instead:
- a public, small-team-runnable, evidence-first qualification layer;
- explicit separation of physical observation, literature value, engineering estimate, model prediction and synthetic fixture;
- exact reproducibility and independent numerical oracles;
- full boundary accounting from plasma to export, availability and economics;
- open cross-device evaluation with frozen holdouts and negative controls;
- auditable design decisions that show which measurements would most reduce risk.

## Paradigm shifts worth pursuing

1. **From headline Q to boundary-complete performance.** A result is not a plant result until the control surface, primary input, all recirculating loads, useful product and grid/product boundary are explicit.
2. **From point estimates to decision-grade distributions.** A nominal Q_econ of 1.1 is not a robust design if plausible uncertainty leaves a high probability of Q_econ below 1.
3. **From maintenance as an afterthought to maintainability as a design variable.** Geometry, access, lifting paths, tool recovery, parallel maintenance and hot-cell logistics should influence the design before hardware is frozen.
4. **From isolated digital twins to evidence-carrying model chains.** A model output should carry data rights, source digest, version, calibration, uncertainty, validation domain and reproducible execution receipt.
5. **From maximizing power density alone to co-optimizing economics and lifetime.** Higher P_f/S can increase economic gain while shortening surface life and lowering utilization; the optimum depends on fluence, replacement time, repair cost, cooling limits and market value.
6. **From a single architecture contest to a common qualification protocol.** Compare tokamak, stellarator, IFE, magneto-inertial, and other concepts with matched system boundaries and architecture-specific constraints—not one borrowed threshold.

## Claim ceiling

Current implementation is a deterministic Q_econ software model with an analytic design-threshold solver, an evidence metadata wrapper, and an adapter from a complete electrical balance. It is not a validated plant twin, stochastic RAMI model, bankable economics model, neutronics transport solver, validated plasma predictor, or demonstration of fusion energy production. All future performance claims must be tied to independent benchmark results and the exact tested revision.
