# Cross-Repository Reactor Research Contract — 2026

**Date:** 2026-11-14  
**Status:** Architecture and evidence contract; no physical reactor validation claimed

## Purpose

Coordinate Spark, Symthaea and Symtropy as one research programme without duplicating physics engines or silently promoting model outputs into physical evidence.

| System | Owns | Does not claim |
|---|---|---|
| Spark Engine | Nuclear-reaction calculations, lattice/material reaction hypotheses, LCF literature corpus, uncertainty and experiment-ranking logic, reactor architecture comparison | A validated reactor, net energy, physical replication, radiation safety or manufacturing authority |
| Symthaea | Hypothesis generation, physics knowledge representation, equation/model discovery, evidence synthesis, causal/model-adequacy reasoning | A validated plasma PDE solver merely because it has plasma encoders or analytic formula helpers |
| Symtropy | Scenario identity, deterministic replay, solver adapters, mechanical/thermal/energy-boundary receipts, evidence artifact integrity | A validated plasma, neutron-transport, CFD, FEM or reactor solver unless a named solver/model has separately passed VVUQ |

## Findings from source review

### Spark Engine

The standalone public crate is the owner of LCF-specific code. The Symthaea vendored copy explicitly states that it is excluded from its workspace and has no application consumers; the bridge hooks are placeholders. Do not treat the vendored copy as a live integration.

The current Spark public research programme is documentation-first. Its 2025 and 2026 literature updates are beam-driven D–D studies in metallic hydrides, not demonstrations of a lattice power reactor.

### Symthaea

The core physics crate contains active spark_engine, spark_prototype_spec, radiation_damage, trigger_systems, material and phonon modules. The spark_engine header correctly calls the v1.0 arrangement a hypothetical target and states that net energy gain is unestablished.

The source tree also contains reactor_architectures.rs and scaling_study.rs, but neither is declared by crates/core/symthaea-core/src/physics/mod.rs on the reviewed main revision. They are therefore not active capabilities through that module. The historical enhancement report must not be used as proof that these source files are compiled or validated.

The active prototype-spec generator can label a model-feasible result as DESIGN VALIDATED. That wording exceeds what a model feasibility flag establishes. Its physical conversion, material cost, lifetime, BOM and manufacturing fields must remain illustrative until each assumption has provenance and independent qualification. A model-only result must never be presented as a physical design release, safety case or regulatory approval.

plasma_physics.rs is documented as exploratory HDC encoding; symthaea-continuum-physics/plasma_mhd.rs provides analytic relations and formula helpers, not a qualified MHD PDE/equilibrium solver. The digital-twin crate stores telemetry and residuals, but is not itself a plasma or nuclear solver.

### Symtropy

symtropy-physics is a rigid-body mechanics engine. Its research validation document explicitly lists production fluid, deformable, fracture, FEM and MPM simulation as not certified. symtropy-fluid currently has placeholder gradient/Laplacian logic and must not be used for design-grade reactor thermal-hydraulics.

Two existing Symtropy workstreams are directly reusable:
- [PLASMA-TWIN-001 #1685](https://github.com/Luminous-Dynamics/symtropy/issues/1685) defines a solver-neutral plasma scenario adapter, including a distinct fusion regime profile. It explicitly keeps the specialist solver responsible for its own equations and results.
- [PHYS-ENERGY-PORTS-001 #3615](https://github.com/Luminous-Dynamics/symtropy/issues/3615) defines dimensionally correct named energy transfers and first-law instrumentation. Reuse this rather than creating a second generic energy ledger.

Related materials-manufacturing work should remain linked but distinct from fusion plasma physics:
- [Symthaea PLASMA-SURFACE-001 #11795](https://github.com/Luminous-Dynamics/symthaea/issues/11795) concerns atmospheric-plasma surface activation and durable bonding.
- [Symtropy SHIPYARD-WELD-PHYSICS-VVUQ-001 #4614](https://github.com/Luminous-Dynamics/symtropy/issues/4614) concerns weld thermal/residual-stress/distortion VVUQ.
These can inform fabrication, joining and materials qualification; they are not evidence for fusion confinement or LCF reaction mechanisms.

## Required cross-system artifact

The smallest useful integration is a versioned, immutable **Reactor Research Run Receipt**. It should be a data contract, not a new solver.

Required fields:
- run ID and schema version;
- repository, full Git commit and build/toolchain identity for each producer;
- immutable scenario/configuration digest;
- architecture and physical regime identifier;
- model and solver identity, exact version/commit, capability profile and declared omissions;
- source/dataset identifiers, DOI or stable URL, license/rights state and content hash;
- values with SI unit, quantity kind, reference frame, time/space support and uncertainty;
- declared system boundary and nested-boundary relationships;
- raw artifact references and canonical output digest;
- calibration and validation evidence references;
- result disposition: measured, externally reported, modelled, extrapolated, unsupported, non-converged, incomplete or unknown;
- claim ceiling and unresolved risks.

A successful receipt integrity check means only that identity, schema and artifact rules passed. It does not prove solver correctness, model validity, physical replication, safety, or device readiness.

## Canonical hand-offs

1. **Symthaea → Spark:** hypotheses, candidate observables, model-adequacy challenges and evidence-linked literature statements. No unproven equation becomes a trusted physical law by being encoded.
2. **Spark → Symtropy:** a versioned scenario plus explicit predictions, parameter distributions, model validity envelope and output quantities. Spark remains responsible for nuclear reaction calculations.
3. **Symtropy → Spark/Symthaea:** deterministic run receipts, energy/material boundary residuals, solver warnings, prediction residuals and validation status. Unknown or failed runs remain visible.
4. **Independent measurement → all three:** immutable observation records with instrument calibration, uncertainty, source identity, physical sample lineage and rights. No model output may be retyped as a measurement.

## Energy metric separation

Every programme report must keep these quantities separate:
- reaction energy and particle yield;
- target gain, when applicable;
- plasma gain, when applicable;
- driver wall-plug gain;
- gross thermal power;
- gross electrical generation;
- auxiliary and recirculating loads;
- net electricity exported or useful process heat delivered.

Do not compare NIF target gain, magnetic-confinement triple product, plasma duration, a beam-target yield ratio and whole-plant net-electric output as if they were one leaderboard.

For a plant boundary, account for energy inputs, energy outputs and change in stored energy. A claim of power production requires positive net electricity at the declared facility boundary with uncertainty bounds that remain positive. A target/plasma gain milestone is not a plant-level result.

## Benchmark and evidence gates

| Gate | Pass criterion |
|---|---|
| Identity | Every input, dataset, model and solver revision is immutable and traceable |
| Dimensional integrity | Unknown units, incompatible frames, non-finite quantities and invalid conversions are rejected |
| Model validity | Predictions are tested against independent references and held-out data inside a declared envelope |
| Uncertainty | Predictive calibration and uncertainty propagation are evaluated, not just point error |
| Reproducibility | Same frozen input and deterministic contract reproduce the same result; nondeterminism is declared |
| Boundary closure | All material and energy flows crossing the declared boundary are named; nested transfers are not double-counted |
| Claim scope | Simulation, external literature, measurement, physical replication and qualification are distinct states |
| Safety/authority | No software receipt substitutes for professional engineering review, licensed facilities, radiation protection or regulatory authorization |

## Immediate implementation sequence

1. Complete Spark's rate/unit reconciliation and the new metal-hydride benchmark manifest.
2. Audit Symthaea's active prototype-spec output so model feasibility cannot produce a physical-validation label; quarantine unproven BOM, cost, efficiency, lifetime and safety assumptions.
3. Decide whether dormant reactor-architecture/scaling files should be activated only after provenance and model validation, or retained as archived exploratory work.
4. Reuse Symtropy #1685 for solver-neutral scenario receipts and #3615 for typed energy ports; add only the nuclear/fusion-specific adapter fields.
5. Add a matched, external reference benchmark before selecting or implementing any new thermal-fluid, MHD or neutron-transport solver.
6. Keep the atmospheric-plasma bonding and weld VVUQ programmes as adjacent manufacturing/materials qualification work, not conflated with fusion plasma.

## Claim boundary

This document is a source-code and architecture audit. It does not report a physical experiment, a qualified digital twin, a validated reactor design, net energy production, or approval to manufacture or operate nuclear/radiation-producing equipment.


## Competitive landscape and differentiation — checked 2026-11-14

The industry is now explicitly investing in AI-enabled fusion digital twins. On 8 October 2026, Commonwealth Fusion Systems announced that it had been selected by the U.S. Department of Energy to lead the Phase II Genesis Mission project for an AI-enabled digital twin of its SPARC machine, with ten national laboratories, universities and industry partners. The DOE's final June 2026 Fusion Science and Technology Roadmap separately calls for an AI-Fusion Digital Convergence Platform to support burning-plasma science, materials discovery, fuel-cycle closure and fusion-energy systems integration.

This means "AI for fusion" or "a fusion digital twin" is not, by itself, a defensible Luminous differentiator. We should not attempt to outbuild a national-laboratory consortium on a single tokamak's operations twin. Instead, pursue a complementary layer:

1. **Cross-architecture evidence model:** one claim, provenance, unit, uncertainty and validation contract that can compare magnetic, inertial, pulsed, materials-mediated and industrial-plasma concepts without pretending their physics metrics are interchangeable.
2. **Model-adequacy and open-world qualification:** detect when every registered model is wrong, preserve failed predictions, and select experiments for robust decision value rather than point expected information gain alone. Reuse Symthaea's existing model-adequacy and robust experiment-design work.
3. **Whole-system boundary assurance:** distinguish reaction yield, target/plasma gain, driver wall-plug efficiency, gross heat/electric output, recirculating loads and net exported electricity. Reuse Symtropy's typed energy-port programme.
4. **Materials-to-component evidence chain:** preserve specimen/lot lineage, microstructure, irradiation/thermal history, joining and inspection evidence, and model-transfer limits. Compose with Symthaea's materials/plasma-bond work and Symtropy's weld VVUQ instead of claiming a new materials solver.
5. **Solver-neutral federation:** orchestrate external specialist solvers and compare them only where equations, boundary conditions and observables are compatible. Never relabel an internal visualization or synthetic projection as a validated field solution.

These are proposed differentiators, not capabilities already demonstrated by Luminous. Our competitive benchmark should be: same frozen public case, same declared system boundary, same reference data and uncertainty; show better held-out prediction or more reliable decision/qualification outcomes, and publish the failed cases as well as the wins.

References:
- [CFS: AI-enabled SPARC digital twin, Genesis Mission Phase II, 8 October 2026](https://cfs.energy/news-and-media/commonwealth-fusion-systems-to-lead-ai-enabled-digital-twin-fusion-project-in-second-phase-of-genesis-mission/)
- [DOE: Final Fusion Science and Technology Roadmap, 9 June 2026](https://www.energy.gov/articles/energy-department-releases-finalized-fusion-science-and-technology-roadmap-accelerate)
- [LLNL: NIF repeated ignition; 7.9 MJ and target gain ~3.8, 20 June 2026](https://lmf.llnl.gov/science/achieving-fusion-ignition)
- [Wendelstein 7-X: triple product result, *Physical Review E* (2026)](https://doi.org/10.1103/q59k-8m26)
