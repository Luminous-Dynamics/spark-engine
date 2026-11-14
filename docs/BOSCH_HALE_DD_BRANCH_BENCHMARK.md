# Bosch–Hale D–D Branch Benchmark and Cross-Validation

**Date:** 2026-11-14  
**Status:** Reference implementation added; Rust tests written but not executed in this environment  
**Module:** `src/bosch_hale.rs`

## Purpose

Spark now has a bare-nucleus D–D reference module that keeps the two reaction branches separate:

- D(d,n)³He — neutron branch.
- D(d,p)T — proton/triton branch.

It implements two independently parameterized Bosch–Hale paths:

1. Energy-dependent branch cross-sections, expressed in millibarns as a function of centre-of-mass energy.
2. Analytic Maxwellian branch reactivities, expressed in cm³/s as a function of ion temperature.

It also numerically integrates each energy-dependent cross-section over a Maxwellian relative-energy distribution and compares the result with the analytic branch-reactivity fit. This is a cross-implementation consistency check, not an experimental validation.

The new module is the bare-nucleus reference path. It contains no lattice screening, phonon, hot-spot or beam-target correction.

## Primary sources and coefficient provenance

1. H.-S. Bosch and G. M. Hale, “Improved formulas for fusion cross-sections and thermal reactivities,” *Nuclear Fusion* 32, 611–631 (1992), DOI [10.1088/0029-5515/32/4/i07](https://doi.org/10.1088/0029-5515/32/4/i07).
2. Z. S. Hartwig and Y. A. Podpaly, *MIT Magnetic Fusion Energy Formulary* (2014), §§9.7–9.8: [formulary PDF](https://library.psfc.mit.edu/catalog/online_pubs/MFE_formulary_2014.pdf). This source prints the branch coefficients, energy validity domains, cross-section table and Maxwellian branch-rate table used by the regression fixtures.
3. A. S. Richardson, *NRL Plasma Formulary* (2019), p. 45: [PDF](https://homepage.physics.uiowa.edu/~ghowes/teach/astr7830/resources/NRL_Plasma_Formulary_2019.pdf). This is an additional total D–D reactivity comparison, not an exact branch-by-branch replacement for the Bosch–Hale fits.

## Units and validity

### Cross-section fits

- Input: centre-of-mass energy in keV.
- Output: millibarns.
- D(d,n)³He validity: 0.5 < E_cm < 4900 keV.
- D(d,p)T validity: 0.5 < E_cm < 5000 keV.

Laboratory beam energy must not be substituted for centre-of-mass energy without an explicit kinematic conversion.

### Maxwellian reactivity fits

- Input: ion temperature in keV.
- Output: cm³/s.
- Published validity: 0.2 < T_i < 100 keV.
- Branch-specific coefficients use the reduced D–D mass parameter of 937,814 keV/c².

The numerical cross-section integration path is deliberately more conservative: 1 ≤ T_i < 100 keV. Its lower bound avoids implying that an integral truncated at the published 0.5 keV cross-section boundary is a complete Maxwellian integral at lower temperatures.

Neither path is validated at 300 K or in a deuterated solid-lattice/beam-target regime.

## Reference cross-sections

MIT Formulary §9.7 tabulates the following cross-sections in millibarns:

| E_cm (keV) | D(d,n)³He (mb) | D(d,p)T (mb) |
|---:|---:|---:|
| 3 | 2.445 × 10⁻⁴ | 2.513 × 10⁻⁴ |
| 4 | 2.093 × 10⁻³ | 2.146 × 10⁻³ |
| 5 | 8.834 × 10⁻³ | 9.038 × 10⁻³ |
| 6 | 2.517 × 10⁻² | 2.569 × 10⁻² |
| 10 | 2.779 × 10⁻¹ | 2.812 × 10⁻¹ |
| 20 | 2.691 | 2.670 |

The module tests compare the polynomial cross-section implementation against these table values with a 0.2% tolerance (the tabulated values are rounded).

## Reference Maxwellian branch rates

MIT Formulary §9.8 tabulates branch reactivities in m³/s. Values below are converted to cm³/s by multiplying by 10⁶.

| T_i (keV) | D(d,n)³He (cm³/s) | D(d,p)T (cm³/s) | Total (cm³/s) |
|---:|---:|---:|---:|
| 1 | 9.933 × 10⁻²³ | 1.017 × 10⁻²² | 2.010 × 10⁻²² |
| 2 | 3.110 × 10⁻²¹ | 3.150 × 10⁻²¹ | 6.260 × 10⁻²¹ |
| 5 | 9.128 × 10⁻²⁰ | 9.024 × 10⁻²⁰ | 1.815 × 10⁻¹⁹ |
| 10 | 6.023 × 10⁻¹⁹ | 5.781 × 10⁻¹⁹ | 1.180 × 10⁻¹⁸ |
| 20 | 2.603 × 10⁻¹⁸ | 2.399 × 10⁻¹⁸ | 5.002 × 10⁻¹⁸ |

The analytic branch formula tests compare against these values at 0.1% tolerance, preserving the table's rounding.

## Independent cross-section integration

The module also evaluates:

[
\langle\sigma v\rangle =
\sqrt{\frac{8}{\pi\mu}}(kT)^{-3/2}
\int \sigma(E) E e^{-E/kT}\,dE
]

The cross-section is in millibarns; the implementation includes the appropriate millibarn-to-cm² conversion and the reduced mass in the Maxwellian prefactor.

For numerical stability, integration is performed in (x=\ln E), with an adaptive Simpson quadrature and an explicit error estimate. The upper cutoff is selected at 60 thermal energies or the cross-section validity bound, whichever is smaller.

The current independent calculation agrees with the analytic Bosch–Hale branch-rate fit to within approximately 2% across 1–50 keV. The Rust tests use a 2.5% acceptance envelope to allow for finite integration-boundary and tabulated fit differences, while separately requiring the quadrature error estimate to be below 10⁻⁷.

This separates:
- numerical integration error;
- cross-section parameterization / reference-fit difference;
- uncertainty in measured nuclear data.

The quadrature estimate is not a physical measurement uncertainty.

## Comparison with NRL 2019

The NRL 2019 Formulary reports total D–D Maxwellian reactivities (1a + 1b) of:

| T_i (keV) | NRL total (cm³/s) |
|---:|---:|
| 1 | 1.5 × 10⁻²² |
| 2 | 5.4 × 10⁻²¹ |
| 5 | 1.8 × 10⁻¹⁹ |
| 10 | 1.2 × 10⁻¹⁸ |
| 20 | 5.2 × 10⁻¹⁸ |

The Bosch–Hale total branch fit differs most at 1 keV (approximately 34% higher) and approaches within a few percent at 5–20 keV. These are distinct published fit/table values; do not silently tune one parameterization to reproduce another. Record the reference model and source explicitly.

The previous 35% NRL acceptance envelope applied to Spark's constant-S approximation only. The new Bosch–Hale implementation is tested against its own branch-resolved coefficients and tables.

## Test inventory

The new Rust module contains tests for:

- branch cross-sections versus published tabulated values;
- branch Maxwellian rates versus published tabulated values;
- total equals neutron plus proton branch;
- numerical integration versus the analytic branch-rate fit;
- quadrature error estimate;
- rejection of energies and temperatures outside their stated validity domains;
- rejection of non-finite input.

**Execution status:** tests have been written but were not run in this environment because no Rust/Cargo toolchain is available. No CI pass or exact-head compilation is claimed.

## Claim boundary

The module qualifies a reference computation within published fit domains. It does not validate:
- room-temperature Maxwellian extrapolation;
- beam-target screening inserted into a Maxwellian;
- lattice-mediated reaction enhancement;
- the NASA X-ray-triggered configuration;
- a neutron source or net-energy reactor.

Use Spark's separate LCF/material evidence corpus for those questions, and retain experiment class, detector/source boundary, uncertainty and model-validity metadata.
