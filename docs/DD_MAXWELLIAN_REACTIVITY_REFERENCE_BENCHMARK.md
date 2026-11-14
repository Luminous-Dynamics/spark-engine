# D-D Maxwellian Reactivity Unit and Reference Benchmark

**Date:** 2026-11-14  
**Status:** Reference regression contract; exact-head Rust tests pending  
**Scope:** Bare-nucleus D-D Maxwellian reactivity in Spark's constant-S Gamow approximation

## What was wrong

The previous integration prefactor was:

`3.727e-12 / (sqrt(mu_amu) * (kT_keV)^1.5)`

For an energy integral with:
- S-factor in keV·barn;
- integration energy in keV;
- cross-section in barns;
- output in cm³/s;
- reduced mass in atomic mass units;

the unit conversion is:

`sqrt(8 / (pi * m_u)) * sqrt(1 keV in erg) * 1 barn`

which evaluates to approximately `4.956792115648147e-17`. The previous coefficient was larger by approximately 7.5×10⁴.

A second convention error was that the code treated approximately 55 keV·barn as the total D-D S-factor and then divided the total reactivity between the two branches. The Bosch–Hale low-energy branch coefficients used by WarpX correspond to approximately 53.701 keV·barn for D(d,n)³He and 55.576 keV·barn for D(d,p)T. Their sum, approximately 109.277 keV·barn, is the constant-S approximation used for this benchmark. This is still an approximation; the full energy-dependent branch fits are more accurate.

## Reference data

The 2019 Naval Research Laboratory Plasma Formulary, p. 45, gives the following D-D Maxwellian reactivities for the sum of the two branches:

| Ion temperature | NRL D-D reactivity |
|---:|---:|
| 1 keV | 1.5 × 10⁻²² cm³/s |
| 2 keV | 5.4 × 10⁻²¹ cm³/s |
| 5 keV | 1.8 × 10⁻¹⁹ cm³/s |
| 10 keV | 1.2 × 10⁻¹⁸ cm³/s |
| 20 keV | 5.2 × 10⁻¹⁸ cm³/s |

With screening and phonon shifts disabled, the corrected constant-S integration gives approximately:

| Ion temperature | Spark approximation | Relative difference |
|---:|---:|---:|
| 1 keV | 1.93 × 10⁻²² cm³/s | +28.7% |
| 2 keV | 5.91 × 10⁻²¹ cm³/s | +9.5% |
| 5 keV | 1.65 × 10⁻¹⁹ cm³/s | −8.6% |
| 10 keV | 1.01 × 10⁻¹⁸ cm³/s | −15.8% |
| 20 keV | 3.89 × 10⁻¹⁸ cm³/s | −25.1% |

The source and integration assumptions justify a 35% regression envelope for this constant-S baseline. This is not a 35% claim about experimental uncertainties; it is an acceptance envelope for a simplified model compared with the NRL table.

## Validation boundary

- The 1–20 keV regression points sit within the NRL Formulary's low-temperature fit domain (T ≲ 25 keV).
- The test disables screening and phonon shifts. It benchmarks the bare-nucleus Maxwellian baseline only.
- The 300 K solid-lattice problem is far below the tabulated thermonuclear reactivity domain. A constant-S extrapolation there is **not** a validated standard reactivity, and a beam-target screening potential must not be silently treated as a Maxwellian plasma screening correction.
- The benchmark does not validate lattice-mediated fusion, X-ray-triggered LCF, a neutron source, net energy, or any reactor.
- The energy-dependent Bosch–Hale branch fits should be the next higher-accuracy reference implementation. The current constant-S model is intentionally kept as a transparent approximation with a bounded benchmark envelope.

## NASA source-rate semantics

Steinetz et al., *Physical Review C* 101, 044610 (2020), report 1.5 ± 0.3 × 10³ and 1.6 ± 0.3 × 10³ fusion-energy neutrons/s for samples TS1575 and TS1576 after correcting detector counts and scaling to the sample location. The article also reports a calculated 1.2 × 10³ neutrons/s total rate for all 16 samples. These are source-location rates, not detector counts per cm².

The paper's Table V gives absolute detector efficiencies of 7 × 10⁻⁶ for EJ-309 and 2 × 10⁻⁶ for stilbene under its experimental geometry and data-processing chain. Do not use those efficiencies as universal detector constants: they are experiment-specific.

Source: [Steinetz et al., Phys. Rev. C 101, 044610 (2020)](https://doi.org/10.1103/PhysRevC.101.044610).

## Required next improvements

1. Implement energy-dependent Bosch–Hale branch cross-sections and integrate each branch independently.
2. Add a matched reference table for branch-specific and total reactivity.
3. Move the <0.5 keV extrapolation to a separately labelled model domain with explicit screening hypotheses and uncertainties.
4. Keep beam-target observables, detector counts, source-location rates, areal rates and Maxwellian volume reactivities as distinct quantities.
5. Revisit all published rate-gap and Q tables only after the new reference implementation has been independently checked.
