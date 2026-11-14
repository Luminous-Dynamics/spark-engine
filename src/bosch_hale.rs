// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
//! Branch-resolved Bosch-Hale D-D cross-sections and Maxwellian reactivities.
//!
//! This is the bare-nucleus reference path. It deliberately contains no
//! lattice screening, phonon, hot-spot, or beam-target corrections.
//!
//! Coefficients and validity domains:
//! - Hartwig & Podpaly, MIT Magnetic Fusion Energy Formulary (2014), §§9.7–9.8:
//!   https://library.psfc.mit.edu/catalog/online_pubs/MFE_formulary_2014.pdf
//! - Bosch & Hale, Nuclear Fusion 32, 611–631 (1992):
//!   https://doi.org/10.1088/0029-5515/32/4/i07
//!
//! Cross-sections are returned in millibarns and use centre-of-mass energy.
//! Maxwellian reactivities are returned in cm^3/s and are valid only for
//! 0.2 < T_i < 100 keV. Cross-section fits have their own branch-specific
//! energy ranges. Out-of-domain inputs are rejected rather than extrapolated.

use std::fmt;

/// D-D reaction branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DDBranch {
    /// D(d,n)³He.
    Neutron,
    /// D(d,p)T.
    Proton,
}

/// Supported fit validity intervals.
pub const CROSS_SECTION_MIN_KEV: f64 = 0.5;
pub const MAXWELLIAN_TEMPERATURE_MIN_KEV: f64 = 0.2;
pub const MAXWELLIAN_TEMPERATURE_MAX_KEV: f64 = 100.0;

/// Numerical or validity error from a Bosch-Hale evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum BoschHaleError {
    NonFiniteEnergy,
    NonFiniteTemperature,
    EnergyOutOfRange {
        branch: DDBranch,
        energy_kev: f64,
        min_exclusive_kev: f64,
        max_exclusive_kev: f64,
    },
    TemperatureOutOfRange {
        temperature_kev: f64,
        min_exclusive_kev: f64,
        max_exclusive_kev: f64,
    },
    InvalidFit,
    InvalidQuadrature,
}

impl fmt::Display for BoschHaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteEnergy => write!(f, "centre-of-mass energy must be finite"),
            Self::NonFiniteTemperature => write!(f, "ion temperature must be finite"),
            Self::EnergyOutOfRange {
                branch,
                energy_kev,
                min_exclusive_kev,
                max_exclusive_kev,
            } => write!(
                f,
                "{branch:?} cross-section energy {energy_kev} keV is outside ({min_exclusive_kev}, {max_exclusive_kev}) keV"
            ),
            Self::TemperatureOutOfRange {
                temperature_kev,
                min_exclusive_kev,
                max_exclusive_kev,
            } => write!(
                f,
                "Maxwellian temperature {temperature_kev} keV is outside ({min_exclusive_kev}, {max_exclusive_kev}) keV"
            ),
            Self::InvalidFit => write!(f, "Bosch-Hale fit produced a non-finite or non-positive value"),
            Self::InvalidQuadrature => write!(f, "Maxwellian cross-section quadrature failed"),
        }
    }
}

impl std::error::Error for BoschHaleError {}

#[derive(Clone, Copy)]
struct CrossSectionFit {
    bg: f64,
    a: [f64; 5],
    b: [f64; 4],
    max_energy_kev: f64,
}

impl CrossSectionFit {
    fn for_branch(branch: DDBranch) -> Self {
        match branch {
            DDBranch::Neutron => Self {
                bg: 31.3970,
                a: [
                    5.3701e4,
                    3.3027e2,
                    -1.2706e-1,
                    2.9327e-5,
                    -2.5151e-9,
                ],
                b: [0.0; 4],
                max_energy_kev: 4900.0,
            },
            DDBranch::Proton => Self {
                bg: 31.3970,
                a: [
                    5.5576e4,
                    2.1054e2,
                    -3.2638e-2,
                    1.4987e-6,
                    1.8181e-10,
                ],
                b: [0.0; 4],
                max_energy_kev: 5000.0,
            },
        }
    }
}

#[derive(Clone, Copy)]
struct MaxwellianFit {
    bg: f64,
    reduced_mass_c2_kev: f64,
    c: [f64; 7],
}

impl MaxwellianFit {
    fn for_branch(branch: DDBranch) -> Self {
        match branch {
            DDBranch::Neutron => Self {
                bg: 31.3970,
                reduced_mass_c2_kev: 937_814.0,
                c: [
                    5.43360e-12,
                    5.85778e-3,
                    7.68222e-3,
                    0.0,
                    -2.96400e-6,
                    0.0,
                    0.0,
                ],
            },
            DDBranch::Proton => Self {
                bg: 31.3970,
                reduced_mass_c2_kev: 937_814.0,
                c: [
                    5.65718e-12,
                    3.41267e-3,
                    1.99167e-3,
                    0.0,
                    1.05060e-5,
                    0.0,
                    0.0,
                ],
            },
        }
    }
}

/// Branch-resolved Maxwellian reactivity result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DDBoschHaleReactivity {
    /// D(d,n)³He reactivity in cm³/s.
    pub neutron_cm3_s: f64,
    /// D(d,p)T reactivity in cm³/s.
    pub proton_cm3_s: f64,
    /// Sum of the two branch reactivities in cm³/s.
    pub total_cm3_s: f64,
    /// Neutron branch fraction of total reactivity.
    pub neutron_fraction: f64,
    /// Proton branch fraction of total reactivity.
    pub proton_fraction: f64,
}

/// Result of the independent numerical integration of a branch cross-section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratedReactivity {
    /// Numerically integrated branch reactivity in cm³/s.
    pub reactivity_cm3_s: f64,
    /// Adaptive-Simpson estimated relative quadrature error.
    pub estimated_relative_error: f64,
    /// Number of integrand evaluations.
    pub function_evaluations: usize,
    /// Integrated centre-of-mass energy interval, in keV.
    pub energy_interval_kev: (f64, f64),
}

/// Evaluate the Bosch-Hale D-D branch cross-section in millibarns.
///
/// energy_cm_kev is the centre-of-mass energy, not the laboratory beam
/// energy. The fit is restricted to its published interval.
pub fn dd_cross_section_mb(
    branch: DDBranch,
    energy_cm_kev: f64,
) -> Result<f64, BoschHaleError> {
    if !energy_cm_kev.is_finite() {
        return Err(BoschHaleError::NonFiniteEnergy);
    }

    let fit = CrossSectionFit::for_branch(branch);
    if energy_cm_kev <= CROSS_SECTION_MIN_KEV
        || energy_cm_kev >= fit.max_energy_kev
    {
        return Err(BoschHaleError::EnergyOutOfRange {
            branch,
            energy_kev: energy_cm_kev,
            min_exclusive_kev: CROSS_SECTION_MIN_KEV,
            max_exclusive_kev: fit.max_energy_kev,
        });
    }

    let sigma_mb = dd_cross_section_mb_unchecked(fit, energy_cm_kev);
    if sigma_mb.is_finite() && sigma_mb > 0.0 {
        Ok(sigma_mb)
    } else {
        Err(BoschHaleError::InvalidFit)
    }
}

/// Evaluate the Bosch-Hale analytic Maxwellian reactivity for one D-D branch.
///
/// Valid only for 0.2 < temperature_kev < 100.0, as stated in the
/// MIT Magnetic Fusion Energy Formulary (2014), section 9.8.
pub fn dd_branch_maxwellian_cm3_s(
    branch: DDBranch,
    temperature_kev: f64,
) -> Result<f64, BoschHaleError> {
    if !temperature_kev.is_finite() {
        return Err(BoschHaleError::NonFiniteTemperature);
    }
    if temperature_kev <= MAXWELLIAN_TEMPERATURE_MIN_KEV
        || temperature_kev >= MAXWELLIAN_TEMPERATURE_MAX_KEV
    {
        return Err(BoschHaleError::TemperatureOutOfRange {
            temperature_kev,
            min_exclusive_kev: MAXWELLIAN_TEMPERATURE_MIN_KEV,
            max_exclusive_kev: MAXWELLIAN_TEMPERATURE_MAX_KEV,
        });
    }

    let fit = MaxwellianFit::for_branch(branch);
    let [c1, c2, c3, c4, c5, c6, c7] = fit.c;
    let t = temperature_kev;
    let denominator = 1.0 + t * (c3 + t * (c5 + t * c7));
    let theta = t / (1.0 - t * (c2 + t * (c4 + t * c6)) / denominator);
    let xi = (fit.bg * fit.bg / (4.0 * theta)).cbrt();
    let rate = c1 * theta
        * (xi / (fit.reduced_mass_c2_kev * t.powi(3))).sqrt()
        * (-3.0 * xi).exp();

    if rate.is_finite() && rate > 0.0 {
        Ok(rate)
    } else {
        Err(BoschHaleError::InvalidFit)
    }
}

/// Evaluate both Bosch-Hale D-D Maxwellian branches and their total.
pub fn dd_maxwellian_cm3_s(
    temperature_kev: f64,
) -> Result<DDBoschHaleReactivity, BoschHaleError> {
    let neutron = dd_branch_maxwellian_cm3_s(DDBranch::Neutron, temperature_kev)?;
    let proton = dd_branch_maxwellian_cm3_s(DDBranch::Proton, temperature_kev)?;
    let total = neutron + proton;
    if !total.is_finite() || total <= 0.0 {
        return Err(BoschHaleError::InvalidFit);
    }
    Ok(DDBoschHaleReactivity {
        neutron_cm3_s: neutron,
        proton_cm3_s: proton,
        total_cm3_s: total,
        neutron_fraction: neutron / total,
        proton_fraction: proton / total,
    })
}

/// Independently integrate a Bosch-Hale cross-section over a Maxwellian
/// relative-energy distribution. This validates the analytic branch-rate
/// fit; it is not a lattice/beam-target model.
pub fn integrate_dd_branch_maxwellian(
    branch: DDBranch,
    temperature_kev: f64,
    relative_tolerance: f64,
) -> Result<IntegratedReactivity, BoschHaleError> {
    // Reuse the published analytic fit's temperature domain for comparison.
    dd_branch_maxwellian_cm3_s(branch, temperature_kev)?;
    if !relative_tolerance.is_finite() || relative_tolerance <= 0.0 {
        return Err(BoschHaleError::InvalidQuadrature);
    }

    let fit = CrossSectionFit::for_branch(branch);
    let e_min = CROSS_SECTION_MIN_KEV;
    let e_max = (60.0 * temperature_kev)
        .max(20.0)
        .min(fit.max_energy_kev * (1.0 - 1.0e-12));
    if e_max <= e_min {
        return Err(BoschHaleError::InvalidQuadrature);
    }

    // Let x = ln(E). Then sigma(E) * E * exp(-E/T) dE becomes
    // sigma(E) * E^2 * exp(-E/T) dx.
    let x_min = e_min.ln();
    let x_max = e_max.ln();
    let integrand = |x: f64| {
        let energy_kev = x.exp();
        let sigma_mb = dd_cross_section_mb_unchecked(fit, energy_kev);
        sigma_mb * energy_kev * energy_kev * (-energy_kev / temperature_kev).exp()
    };

    let a = x_min;
    let b = x_max;
    let m = 0.5 * (a + b);
    let fa = integrand(a);
    let fm = integrand(m);
    let fb = integrand(b);
    let whole = simpson(a, b, fa, fm, fb);
    let tolerance = relative_tolerance * whole.abs().max(f64::MIN_POSITIVE);
    let mut evaluations = 3usize;
    let (integral_mb_kev3, estimated_abs_error) = adaptive_simpson(
        &integrand,
        a,
        b,
        fa,
        fm,
        fb,
        whole,
        tolerance,
        24,
        &mut evaluations,
    );
    if !integral_mb_kev3.is_finite() || integral_mb_kev3 <= 0.0 {
        return Err(BoschHaleError::InvalidQuadrature);
    }

    // Maxwellian prefactor for sigma(E) in barns would be 4.95679e-17.
    // Here sigma(E) is in millibarns, so multiply the coefficient by 1e-3.
    const REDUCED_MASS_C2_KEV: f64 = 937_814.0;
    const ATOMIC_MASS_ENERGY_KEV: f64 = 931_494.102_42;
    const PREFACTOR_MB_CGS: f64 = 4.956_792_115_648_147e-20;
    let reduced_mass_amu = REDUCED_MASS_C2_KEV / ATOMIC_MASS_ENERGY_KEV;
    let prefactor = PREFACTOR_MB_CGS
        / (reduced_mass_amu.sqrt() * temperature_kev.powf(1.5));
    let reactivity = prefactor * integral_mb_kev3;
    if !reactivity.is_finite() || reactivity <= 0.0 {
        return Err(BoschHaleError::InvalidQuadrature);
    }

    Ok(IntegratedReactivity {
        reactivity_cm3_s: reactivity,
        estimated_relative_error: estimated_abs_error / integral_mb_kev3,
        function_evaluations: evaluations,
        energy_interval_kev: (e_min, e_max),
    })
}

fn dd_cross_section_mb_unchecked(fit: CrossSectionFit, energy_kev: f64) -> f64 {
    let e = energy_kev;
    let s_numerator =
        fit.a[0] + e * (fit.a[1] + e * (fit.a[2] + e * (fit.a[3] + e * fit.a[4])));
    let s_denominator =
        1.0 + e * (fit.b[0] + e * (fit.b[1] + e * (fit.b[2] + e * fit.b[3])));
    let s_mb_kev = s_numerator / s_denominator;
    s_mb_kev / (e * (fit.bg / e.sqrt()).exp())
}

fn simpson(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

#[allow(clippy::too_many_arguments)]
fn adaptive_simpson<F>(
    f: &F,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    whole: f64,
    tolerance: f64,
    depth: u32,
    evaluations: &mut usize,
) -> (f64, f64)
where
    F: Fn(f64) -> f64,
{
    let midpoint = 0.5 * (a + b);
    let left_midpoint = 0.5 * (a + midpoint);
    let right_midpoint = 0.5 * (midpoint + b);
    let flm = f(left_midpoint);
    let frm = f(right_midpoint);
    *evaluations += 2;

    let left = simpson(a, midpoint, fa, flm, fm);
    let right = simpson(midpoint, b, fm, frm, fb);
    let delta = left + right - whole;
    let error_estimate = delta.abs() / 15.0;
    if depth == 0 || error_estimate <= tolerance {
        return (left + right + delta / 15.0, error_estimate);
    }

    let (left_value, left_error) = adaptive_simpson(
        f,
        a,
        midpoint,
        fa,
        flm,
        fm,
        left,
        tolerance * 0.5,
        depth - 1,
        evaluations,
    );
    let (right_value, right_error) = adaptive_simpson(
        f,
        midpoint,
        b,
        fm,
        frm,
        fb,
        tolerance * 0.5,
        depth - 1,
        evaluations,
    );
    (left_value + right_value, left_error + right_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_error(actual: f64, expected: f64) -> f64 {
        (actual - expected).abs() / expected.abs()
    }

    #[test]
    fn cross_sections_match_mfe_formulary_table() {
        let references = [
            (3.0, 2.445e-4, 2.513e-4),
            (4.0, 2.093e-3, 2.146e-3),
            (5.0, 8.834e-3, 9.038e-3),
            (6.0, 2.517e-2, 2.569e-2),
            (10.0, 2.779e-1, 2.812e-1),
            (20.0, 2.691, 2.670),
        ];
        for (energy, expected_neutron, expected_proton) in references {
            let neutron = dd_cross_section_mb(DDBranch::Neutron, energy).unwrap();
            let proton = dd_cross_section_mb(DDBranch::Proton, energy).unwrap();
            assert!(relative_error(neutron, expected_neutron) < 0.002);
            assert!(relative_error(proton, expected_proton) < 0.002);
        }
    }

    #[test]
    fn branch_reactivities_match_mfe_bosch_hale_table() {
        // MIT MFE Formulary §9.8 tabulates m³/s; convert to cm³/s by ×1e6.
        let references = [
            (1.0, 9.933e-23, 1.017e-22),
            (2.0, 3.110e-21, 3.150e-21),
            (5.0, 9.128e-20, 9.024e-20),
            (10.0, 6.023e-19, 5.781e-19),
            (20.0, 2.603e-18, 2.399e-18),
        ];
        for (temperature, expected_neutron, expected_proton) in references {
            let result = dd_maxwellian_cm3_s(temperature).unwrap();
            assert!(relative_error(result.neutron_cm3_s, expected_neutron) < 0.001);
            assert!(relative_error(result.proton_cm3_s, expected_proton) < 0.001);
            assert!((result.total_cm3_s
                - result.neutron_cm3_s
                - result.proton_cm3_s)
                .abs()
                < result.total_cm3_s * 1.0e-14);
        }
    }

    #[test]
    fn numerical_cross_section_integral_matches_analytic_branch_rate_fit() {
        for temperature in [1.0, 2.0, 5.0, 10.0, 20.0, 50.0] {
            for branch in [DDBranch::Neutron, DDBranch::Proton] {
                let analytic =
                    dd_branch_maxwellian_cm3_s(branch, temperature).unwrap();
                let integrated =
                    integrate_dd_branch_maxwellian(branch, temperature, 1.0e-9).unwrap();
                assert!(
                    relative_error(integrated.reactivity_cm3_s, analytic) < 0.025,
                    "{branch:?} at {temperature} keV: integrated={}, analytic={}, estimated quadrature error={}",
                    integrated.reactivity_cm3_s,
                    analytic,
                    integrated.estimated_relative_error
                );
                assert!(integrated.estimated_relative_error < 1.0e-7);
            }
        }
    }

    #[test]
    fn fit_domains_fail_closed() {
        assert!(matches!(
            dd_cross_section_mb(DDBranch::Neutron, 0.5),
            Err(BoschHaleError::EnergyOutOfRange { .. })
        ));
        assert!(matches!(
            dd_branch_maxwellian_cm3_s(DDBranch::Neutron, 0.2),
            Err(BoschHaleError::TemperatureOutOfRange { .. })
        ));
        assert!(matches!(
            dd_branch_maxwellian_cm3_s(DDBranch::Proton, 100.0),
            Err(BoschHaleError::TemperatureOutOfRange { .. })
        ));
    }

    #[test]
    fn total_branch_fraction_sums_to_one() {
        for temperature in [0.5, 1.0, 5.0, 20.0, 99.0] {
            let result = dd_maxwellian_cm3_s(temperature).unwrap();
            assert!((result.neutron_fraction + result.proton_fraction - 1.0).abs() < 1.0e-14);
            assert!(result.neutron_fraction > 0.0);
            assert!(result.proton_fraction > 0.0);
        }
    }

    #[test]
    fn non_finite_inputs_are_rejected() {
        assert_eq!(
            dd_cross_section_mb(DDBranch::Neutron, f64::NAN),
            Err(BoschHaleError::NonFiniteEnergy)
        );
        assert_eq!(
            dd_branch_maxwellian_cm3_s(DDBranch::Neutron, f64::INFINITY),
            Err(BoschHaleError::NonFiniteTemperature)
        );
    }
}
