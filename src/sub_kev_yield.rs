//! Evidence-aware sub-keV D-D yield records and ALR screening comparisons.
//!
//! Kept separate from thermal Gamow-rate and reactor-power calculations.
//! Relative yields and screening enhancements are dimensionless observables or
//! model ratios; they are not absolute reaction rates, Q values or net power.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubKevHostMaterial {
    Palladium,
    Titanium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadingCondition {
    BeamOnly,
    BeamPlusElectrochemicalLoading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionChannel {
    Proton,
    Neutron,
}

/// Figure-digitized and synthetic values are intentionally not raw observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YieldDataOrigin {
    RawDataset,
    FigureDigitized,
    LiteratureReported,
    SyntheticTest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetAvailability {
    Embargoed { release_date: String },
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YieldProvenance {
    pub publication_doi: String,
    pub dataset_doi: String,
    pub dataset_version: Option<String>,
    pub availability: DatasetAvailability,
    pub origin: YieldDataOrigin,
    /// SHA-256 of the exact source artifact; mandatory for RawDataset.
    pub source_artifact_sha256: Option<String>,
    /// Dataset license, not the publication license.
    pub data_license: Option<String>,
}

impl YieldProvenance {
    pub fn validate(&self) -> Result<(), String> {
        if self.publication_doi.trim().is_empty() || self.dataset_doi.trim().is_empty() {
            return Err("publication_doi and dataset_doi must be non-empty".into());
        }
        if self
            .dataset_version
            .as_ref()
            .is_some_and(|v| v.trim().is_empty())
        {
            return Err("dataset_version must not be empty when present".into());
        }
        if self.origin == YieldDataOrigin::RawDataset {
            if self.availability != DatasetAvailability::Released {
                return Err("raw dataset cannot be ingested while the deposit is embargoed".into());
            }
            let digest = self
                .source_artifact_sha256
                .as_deref()
                .ok_or_else(|| "raw dataset requires a source SHA-256".to_string())?;
            if digest.len() != 64 || !digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err("source_artifact_sha256 must be 64 hexadecimal characters".into());
            }
        }
        Ok(())
    }
}

/// A dimensionless thick-target yield normalized to the reference point declared
/// by its containing series. It is not an absolute cross-section or rate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelativeYieldPoint {
    pub energy_cm_kev: f64,
    pub relative_yield: f64,
    /// Marginal ratio uncertainty. The shared reference uncertainty induces
    /// covariance between points and must be modelled separately.
    pub relative_uncertainty: Option<f64>,
    pub counts: Option<u64>,
    pub live_time_s: Option<f64>,
    pub observation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelativeYieldSeries {
    pub series_id: String,
    pub material: SubKevHostMaterial,
    pub loading_condition: LoadingCondition,
    pub channel: DetectionChannel,
    pub reference_energy_cm_kev: f64,
    pub reference_observation_id: String,
    /// Relative uncertainty in the shared reference denominator.
    pub reference_relative_uncertainty: Option<f64>,
    pub provenance: YieldProvenance,
    pub points: Vec<RelativeYieldPoint>,
}

impl RelativeYieldSeries {
    pub fn validate(&self) -> Result<(), String> {
        self.provenance.validate()?;
        if self.series_id.trim().is_empty() || self.reference_observation_id.trim().is_empty() {
            return Err("series_id and reference_observation_id must be non-empty".into());
        }
        if !self.reference_energy_cm_kev.is_finite() || self.reference_energy_cm_kev <= 0.0 {
            return Err("reference_energy_cm_kev must be finite and positive".into());
        }
        if self
            .reference_relative_uncertainty
            .is_some_and(|u| !u.is_finite() || u < 0.0)
        {
            return Err("reference uncertainty must be finite and non-negative".into());
        }
        if self.points.is_empty() {
            return Err("yield series must contain at least one point".into());
        }
        let mut previous = 0.0;
        for point in &self.points {
            if !point.energy_cm_kev.is_finite() || point.energy_cm_kev <= previous {
                return Err(
                    "point energies must be finite, positive and strictly increasing".into(),
                );
            }
            previous = point.energy_cm_kev;
            if !point.relative_yield.is_finite() || point.relative_yield <= 0.0 {
                return Err("relative_yield must be finite and positive".into());
            }
            if point
                .relative_uncertainty
                .is_some_and(|u| !u.is_finite() || u < 0.0)
            {
                return Err("point uncertainty must be finite and non-negative".into());
            }
            if point
                .live_time_s
                .is_some_and(|t| !t.is_finite() || t <= 0.0)
            {
                return Err("live_time_s must be finite and positive".into());
            }
            if point.observation_id.trim().is_empty() {
                return Err("observation_id must be non-empty".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreeningParameterStatus {
    PublishedFit,
    IndependentlyRefitted,
    ModelAssumption,
}

/// High-energy constant-screening fit. It is a condition-specific fitted
/// parameter, not a universal material constant or microscopic measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HomogeneousScreeningFit {
    pub material: SubKevHostMaterial,
    pub loading_condition: LoadingCondition,
    pub screening_potential_kev: f64,
    pub uncertainty_kev: f64,
    pub fit_energy_min_kev: f64,
    pub fit_energy_max_kev: f64,
    pub source_doi: String,
    pub parameter_status: ScreeningParameterStatus,
}

/// The four high-energy homogeneous-fit values reported in Karahadian et al.
/// (2026), each fitted over E_cm = 2.5–6.5 keV.
pub fn published_high_energy_fits() -> [HomogeneousScreeningFit; 4] {
    let source_doi = "10.1038/s41467-026-74421-1".to_string();
    let make = |material, loading_condition, screening_potential_kev| HomogeneousScreeningFit {
        material,
        loading_condition,
        screening_potential_kev,
        uncertainty_kev: 0.2,
        fit_energy_min_kev: 2.5,
        fit_energy_max_kev: 6.5,
        source_doi: source_doi.clone(),
        parameter_status: ScreeningParameterStatus::PublishedFit,
    };
    [
        make(
            SubKevHostMaterial::Palladium,
            LoadingCondition::BeamOnly,
            0.8,
        ),
        make(
            SubKevHostMaterial::Palladium,
            LoadingCondition::BeamPlusElectrochemicalLoading,
            1.7,
        ),
        make(
            SubKevHostMaterial::Titanium,
            LoadingCondition::BeamOnly,
            0.3,
        ),
        make(
            SubKevHostMaterial::Titanium,
            LoadingCondition::BeamPlusElectrochemicalLoading,
            0.9,
        ),
    ]
}

/// ALR homogeneous-screening enhancement over a bare-nucleus yield.
///
/// All energies are keV. E_G is the Gamow energy in keV:
///
/// sqrt(E / (E + Ue)) * exp(sqrt(E_G / E) - sqrt(E_G / (E + Ue))).
///
/// This is dimensionless and must not be interpreted as an absolute reaction
/// rate, electrical output, Q value or energy-production claim.
pub fn alr_screening_enhancement(
    energy_cm_kev: f64,
    screening_potential_kev: f64,
    gamow_energy_kev: f64,
) -> Result<f64, String> {
    validate_alr_inputs(energy_cm_kev, screening_potential_kev, gamow_energy_kev)?;
    if screening_potential_kev == 0.0 {
        return Ok(1.0);
    }
    let shifted = energy_cm_kev + screening_potential_kev;
    let log_value = 0.5 * (energy_cm_kev / shifted).ln()
        + (gamow_energy_kev / energy_cm_kev).sqrt()
        - (gamow_energy_kev / shifted).sqrt();
    let value = log_value.exp();
    if value.is_finite() && value >= 1.0 {
        Ok(value)
    } else {
        Err("ALR enhancement overflowed or fell below unity".into())
    }
}

/// Surface/bulk weighted ALR mixture from the 2026 paper's phenomenological
/// two-environment representation. This is not a unique microscopic theory.
pub fn two_environment_enhancement(
    energy_cm_kev: f64,
    gamow_energy_kev: f64,
    surface_weight: f64,
    surface_screening_kev: f64,
    bulk_screening_kev: f64,
) -> Result<f64, String> {
    validate_alr_inputs(energy_cm_kev, surface_screening_kev, gamow_energy_kev)?;
    if !surface_weight.is_finite() || !(0.0..=1.0).contains(&surface_weight) {
        return Err("surface_weight must be finite and in [0, 1]".into());
    }
    if !bulk_screening_kev.is_finite() || bulk_screening_kev < 0.0 {
        return Err("bulk_screening_kev must be finite and non-negative".into());
    }
    let surface =
        alr_screening_enhancement(energy_cm_kev, surface_screening_kev, gamow_energy_kev)?;
    let bulk = alr_screening_enhancement(energy_cm_kev, bulk_screening_kev, gamow_energy_kev)?;
    let value = surface_weight * surface + (1.0 - surface_weight) * bulk;
    if value.is_finite() && value >= 1.0 {
        Ok(value)
    } else {
        Err("two-environment enhancement is non-finite or below unity".into())
    }
}

/// Numerically invert the homogeneous ALR expression. This returns an
/// equivalent effective Ue for a dimensionless enhancement, not a measurement
/// of the underlying microscopic screening potential.
pub fn equivalent_homogeneous_screening_kev(
    energy_cm_kev: f64,
    enhancement: f64,
    gamow_energy_kev: f64,
) -> Result<f64, String> {
    validate_alr_inputs(energy_cm_kev, 0.0, gamow_energy_kev)?;
    if !enhancement.is_finite() || enhancement < 1.0 {
        return Err("enhancement must be finite and at least 1".into());
    }
    if (enhancement - 1.0).abs() <= f64::EPSILON {
        return Ok(0.0);
    }

    let (mut low, mut high) = (0.0, 1.0);
    let mut found = false;
    for _ in 0..128 {
        if alr_screening_enhancement(energy_cm_kev, high, gamow_energy_kev)? >= enhancement {
            found = true;
            break;
        }
        high *= 2.0;
    }
    if !found {
        return Err("could not bracket equivalent screening potential".into());
    }
    for _ in 0..128 {
        let middle = 0.5 * (low + high);
        if alr_screening_enhancement(energy_cm_kev, middle, gamow_energy_kev)? < enhancement {
            low = middle;
        } else {
            high = middle;
        }
        if high - low <= 1e-12 * high.max(1.0) {
            break;
        }
    }
    Ok(0.5 * (low + high))
}

fn validate_alr_inputs(e: f64, u: f64, eg: f64) -> Result<(), String> {
    if !e.is_finite() || e <= 0.0 {
        return Err("energy_cm_kev must be finite and positive".into());
    }
    if !u.is_finite() || u < 0.0 {
        return Err("screening_potential_kev must be finite and non-negative".into());
    }
    if !eg.is_finite() || eg <= 0.0 {
        return Err("gamow_energy_kev must be finite and positive".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const DD_GAMOW_ENERGY_KEV: f64 = 31.38 * 31.38;

    fn provenance(origin: YieldDataOrigin) -> YieldProvenance {
        YieldProvenance {
            publication_doi: "10.1038/s41467-026-74421-1".into(),
            dataset_doi: "10.5281/zenodo.20435774".into(),
            dataset_version: Some("v1".into()),
            availability: DatasetAvailability::Embargoed {
                release_date: "2026-12-01".into(),
            },
            origin,
            source_artifact_sha256: None,
            data_license: None,
        }
    }

    #[test]
    fn bare_limit_is_unity() {
        assert_eq!(
            alr_screening_enhancement(0.25, 0.0, DD_GAMOW_ENERGY_KEV).unwrap(),
            1.0
        );
    }

    #[test]
    fn stronger_screening_increases_enhancement_at_fixed_energy() {
        let a = alr_screening_enhancement(1.0, 0.3, DD_GAMOW_ENERGY_KEV).unwrap();
        let b = alr_screening_enhancement(1.0, 1.7, DD_GAMOW_ENERGY_KEV).unwrap();
        assert!(b > a && a > 1.0);
    }

    #[test]
    fn two_environment_endpoints_equal_the_selected_component() {
        let surface = alr_screening_enhancement(0.5, 2.0, DD_GAMOW_ENERGY_KEV).unwrap();
        let bulk = alr_screening_enhancement(0.5, 0.4, DD_GAMOW_ENERGY_KEV).unwrap();
        assert!(
            (two_environment_enhancement(0.5, DD_GAMOW_ENERGY_KEV, 1.0, 2.0, 0.4).unwrap()
                - surface)
                .abs()
                < 1e-12
        );
        assert!(
            (two_environment_enhancement(0.5, DD_GAMOW_ENERGY_KEV, 0.0, 2.0, 0.4).unwrap() - bulk)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn equivalent_screening_round_trips_two_environment_result() {
        let combined =
            two_environment_enhancement(0.5, DD_GAMOW_ENERGY_KEV, 0.35, 2.8, 0.8).unwrap();
        let equivalent =
            equivalent_homogeneous_screening_kev(0.5, combined, DD_GAMOW_ENERGY_KEV).unwrap();
        let recovered = alr_screening_enhancement(0.5, equivalent, DD_GAMOW_ENERGY_KEV).unwrap();
        assert!((recovered / combined - 1.0).abs() < 1e-9);
    }

    #[test]
    fn alr_rejects_invalid_inputs() {
        assert!(alr_screening_enhancement(0.0, 0.5, DD_GAMOW_ENERGY_KEV).is_err());
        assert!(alr_screening_enhancement(0.5, -0.1, DD_GAMOW_ENERGY_KEV).is_err());
        assert!(two_environment_enhancement(0.5, DD_GAMOW_ENERGY_KEV, 1.1, 1.0, 0.5).is_err());
    }

    #[test]
    fn embargoed_data_cannot_be_claimed_as_ingested_raw_data() {
        let mut p = provenance(YieldDataOrigin::RawDataset);
        p.source_artifact_sha256 = Some("a".repeat(64));
        assert!(p.validate().is_err());
    }

    #[test]
    fn raw_data_requires_released_status_and_sha256() {
        let mut p = provenance(YieldDataOrigin::RawDataset);
        p.availability = DatasetAvailability::Released;
        assert!(p.validate().is_err());
        p.source_artifact_sha256 = Some("a".repeat(64));
        assert!(p.validate().is_ok());
    }

    #[test]
    fn yield_series_validates_and_retains_common_reference_uncertainty() {
        let series = RelativeYieldSeries {
            series_id: "synthetic-test-fixture".into(),
            material: SubKevHostMaterial::Palladium,
            loading_condition: LoadingCondition::BeamOnly,
            channel: DetectionChannel::Proton,
            reference_energy_cm_kev: 6.5,
            reference_observation_id: "synthetic-reference".into(),
            reference_relative_uncertainty: Some(0.03),
            provenance: provenance(YieldDataOrigin::SyntheticTest),
            points: vec![
                RelativeYieldPoint {
                    energy_cm_kev: 0.25,
                    relative_yield: 0.01,
                    relative_uncertainty: Some(0.002),
                    counts: None,
                    live_time_s: None,
                    observation_id: "synthetic-0".into(),
                },
                RelativeYieldPoint {
                    energy_cm_kev: 6.5,
                    relative_yield: 1.0,
                    relative_uncertainty: Some(0.03),
                    counts: None,
                    live_time_s: None,
                    observation_id: "synthetic-1".into(),
                },
            ],
        };
        assert!(series.validate().is_ok());
        assert_eq!(series.provenance.origin, YieldDataOrigin::SyntheticTest);
        assert_eq!(series.reference_relative_uncertainty, Some(0.03));
    }

    #[test]
    fn published_high_energy_fits_keep_material_loading_and_fit_window() {
        let fits = published_high_energy_fits();
        assert_eq!(fits.len(), 4);
        assert!(
            fits.iter()
                .all(|f| f.fit_energy_min_kev == 2.5 && f.fit_energy_max_kev == 6.5)
        );
        assert!(fits.iter().all(|f| f.uncertainty_kev == 0.2));
        assert_eq!(fits[0].screening_potential_kev, 0.8);
        assert_eq!(fits[1].screening_potential_kev, 1.7);
        assert_eq!(fits[2].screening_potential_kev, 0.3);
        assert_eq!(fits[3].screening_potential_kev, 0.9);
    }
}
