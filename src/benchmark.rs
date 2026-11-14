// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
//! Provenance- and unit-aware records for external materials-mediated fusion
//! benchmark observations.
//!
//! This module intentionally defines a data contract, not a reaction-rate
//! model. It does not invent pointwise data from published figures. Each
//! observation must identify its experiment class, source locator, observable,
//! normalization basis, energy frame, uncertainty status, and data-rights state.
//!
//! In particular, a dimensionless yield enhancement relative to a bare-nucleus
//! model is not a power measurement, a Q value, or evidence of net energy gain.

use serde::{Deserialize, Serialize};

/// How a record entered the benchmark corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceClass {
    /// A value reported in a publication; raw pointwise data may not be present.
    ExternallyReported,
    /// A value taken from a source dataset with a stable content digest.
    RawDataset,
    /// A value digitized from a published figure.
    DigitizedFigure,
    /// A generated fixture used to test the schema or evaluator.
    SyntheticFixture,
    /// A result produced by a named computational model.
    ModelOutput,
}

/// Rights/access state for the source artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataRights {
    Open,
    PaperOnly,
    Restricted,
    RightsPending,
    Unknown,
}

/// Reactor/material experiment classes that must not be silently conflated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentClass {
    /// Incident deuterons drive reactions in a metal or metal-hydride target.
    BeamDrivenMetalTarget,
    /// A lattice experiment with an X-ray trigger; not equivalent to ion-beam loading.
    XrayTriggeredLattice,
    /// Electrochemical loading/control without a declared incident-ion beam.
    ElectrochemicalLoadingControl,
    /// Other experiment class; details belong in the source locator/notes.
    Other,
}

/// Material loading state at the time represented by the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadingState {
    BeamOnly,
    ElectrochemicallyLoaded,
    GasLoaded,
    Unknown,
}

/// Detection channel associated with the observable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionChannel {
    Proton,
    Neutron,
    Calorimetry,
    ChargedParticleOther,
    Other,
}

/// Energy reference frame. Beam energy and centre-of-mass energy are not interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyFrame {
    Laboratory,
    CenterOfMass,
}

/// Physical kind of a reported benchmark quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantityKind {
    IncidentEnergy,
    RelativeYield,
    HydrideToMetalYieldRatio,
    AbsoluteYield,
    DetectorCountRate,
    ArealDetectorCountRate,
    TotalReactionRate,
    VolumetricReactionRate,
    CalorimetricPower,
    LoadingRatio,
    EnhancementRatio,
}

/// Explicit units supported by this first benchmark contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementUnit {
    Dimensionless,
    ElectronVolt,
    KiloElectronVolt,
    EventsPerIncidentIon,
    CountsPerSecond,
    CountsPerSecondPerSquareCentimeter,
    ReactionsPerSecond,
    ReactionsPerSecondPerCubicCentimeter,
    Watt,
    DeuteriumPerHostAtom,
}

/// Whether the uncertainty is reported, absent from the source, or not applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UncertaintyStatus {
    Reported,
    NotReported,
    NotApplicable,
}

/// Meaning of a measurement's numeric value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueBasis {
    /// A value with an absolute physical interpretation under its declared unit.
    Absolute,
    /// A relative value normalized to a named reference observation.
    NormalizedTo { reference_id: String },
    /// A ratio to a named experimental control.
    RatioToControl { control_id: String },
    /// An enhancement ratio relative to a named model/reference calculation.
    EnhancementVsModel { model_id: String },
}

/// Statistical interpretation of a reported uncertainty magnitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UncertaintyKind {
    StandardUncertainty,
    StandardDeviation,
    ExpandedUncertainty,
}

/// Explicit uncertainty metadata. The magnitude uses the same unit as its measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementUncertainty {
    pub magnitude: f64,
    pub kind: UncertaintyKind,
    /// Required for expanded uncertainty; e.g. 0.95 for a 95% coverage probability.
    pub coverage_probability: Option<f64>,
}

/// One measurement with an explicit physical meaning and normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub quantity: QuantityKind,
    pub value: f64,
    pub unit: MeasurementUnit,
    pub basis: ValueBasis,
    pub uncertainty_status: UncertaintyStatus,
    pub uncertainty: Option<MeasurementUncertainty>,
    /// Required for incident energy; forbidden for non-energy observables.
    pub energy_frame: Option<EnergyFrame>,
}

impl Measurement {
    /// Validate dimensional semantics, normalization and uncertainty completeness.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.value.is_finite() || self.value < 0.0 {
            errors.push("measurement value must be finite and non-negative".to_string());
        }

        let unit_matches = match self.quantity {
            QuantityKind::IncidentEnergy => matches!(
                self.unit,
                MeasurementUnit::ElectronVolt | MeasurementUnit::KiloElectronVolt
            ),
            QuantityKind::RelativeYield
            | QuantityKind::HydrideToMetalYieldRatio
            | QuantityKind::EnhancementRatio => self.unit == MeasurementUnit::Dimensionless,
            QuantityKind::AbsoluteYield => {
                self.unit == MeasurementUnit::EventsPerIncidentIon
            }
            QuantityKind::DetectorCountRate => {
                self.unit == MeasurementUnit::CountsPerSecond
            }
            QuantityKind::ArealDetectorCountRate => {
                self.unit == MeasurementUnit::CountsPerSecondPerSquareCentimeter
            }
            QuantityKind::TotalReactionRate => {
                self.unit == MeasurementUnit::ReactionsPerSecond
            }
            QuantityKind::VolumetricReactionRate => {
                self.unit == MeasurementUnit::ReactionsPerSecondPerCubicCentimeter
            }
            QuantityKind::CalorimetricPower => self.unit == MeasurementUnit::Watt,
            QuantityKind::LoadingRatio => {
                self.unit == MeasurementUnit::DeuteriumPerHostAtom
            }
        };
        if !unit_matches {
            errors.push(format!(
                "unit {:?} is incompatible with quantity {:?}",
                self.unit, self.quantity
            ));
        }

        let basis_matches = match self.quantity {
            QuantityKind::RelativeYield => {
                matches!(&self.basis, ValueBasis::NormalizedTo { reference_id } if !reference_id.trim().is_empty())
            }
            QuantityKind::HydrideToMetalYieldRatio => {
                matches!(&self.basis, ValueBasis::RatioToControl { control_id } if !control_id.trim().is_empty())
            }
            QuantityKind::EnhancementRatio => {
                matches!(&self.basis, ValueBasis::EnhancementVsModel { model_id } if !model_id.trim().is_empty())
            }
            _ => matches!(&self.basis, ValueBasis::Absolute),
        };
        if !basis_matches {
            errors.push(format!(
                "basis {:?} is incompatible with quantity {:?}",
                self.basis, self.quantity
            ));
        }

        if self.quantity == QuantityKind::IncidentEnergy {
            if self.energy_frame.is_none() {
                errors.push("incident energy requires an explicit reference frame".to_string());
            }
        } else if self.energy_frame.is_some() {
            errors.push("energy frame is only valid for incident energy".to_string());
        }

        match (self.uncertainty_status, &self.uncertainty) {
            (UncertaintyStatus::Reported, None) => errors.push(
                "uncertainty is marked reported but no uncertainty value is present".to_string(),
            ),
            (UncertaintyStatus::NotReported | UncertaintyStatus::NotApplicable, Some(_)) => {
                errors.push(
                    "uncertainty value is present but status says it is absent or not applicable"
                        .to_string(),
                );
            }
            (_, Some(uncertainty)) => {
                if !uncertainty.magnitude.is_finite() || uncertainty.magnitude < 0.0 {
                    errors.push(
                        "uncertainty magnitude must be finite and non-negative".to_string(),
                    );
                }
                match uncertainty.kind {
                    UncertaintyKind::ExpandedUncertainty => {
                        match uncertainty.coverage_probability {
                            Some(p) if p.is_finite() && p > 0.0 && p < 1.0 => {}
                            _ => errors.push(
                                "expanded uncertainty requires a coverage probability in (0, 1)"
                                    .to_string(),
                            ),
                        }
                    }
                    _ if uncertainty.coverage_probability.is_some() => errors.push(
                        "coverage probability is only valid for expanded uncertainty".to_string(),
                    ),
                    _ => {}
                }
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Stable publication/dataset provenance for a benchmark point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    pub citation: String,
    pub doi: Option<String>,
    pub stable_url: Option<String>,
    /// Figure, table, page, dataset file, or model-output identifier.
    pub source_locator: String,
    /// SHA-256 of the exact raw/digitized artifact when one is available.
    pub content_sha256: Option<String>,
    pub rights: DataRights,
}

impl SourceReference {
    pub fn validate(&self, evidence_class: EvidenceClass) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.citation.trim().is_empty() {
            errors.push("source citation must not be empty".to_string());
        }
        if self.source_locator.trim().is_empty() {
            errors.push("source locator must identify a figure, table, page, or artifact".to_string());
        }

        let has_doi = self
            .doi
            .as_deref()
            .is_some_and(|doi| doi.trim().starts_with("10.") && doi.contains('/'));
        let has_url = self.stable_url.as_deref().is_some_and(|url| {
            url.trim().starts_with("https://") || url.trim().starts_with("http://")
        });
        let has_digest = self.content_sha256.is_some();
        if evidence_class != EvidenceClass::SyntheticFixture
            && !has_doi
            && !has_url
            && !has_digest
        {
            errors.push("source requires a DOI, stable URL, or content digest".to_string());
        }

        if let Some(digest) = &self.content_sha256 {
            let valid_digest = digest.len() == 64
                && digest.bytes().all(|byte| byte.is_ascii_hexdigit());
            if !valid_digest {
                errors.push("content_sha256 must be 64 hexadecimal characters".to_string());
            }
        }

        if matches!(evidence_class, EvidenceClass::RawDataset | EvidenceClass::DigitizedFigure)
            && !has_digest
        {
            errors.push(
                "raw-dataset and digitized-figure evidence require a content SHA-256".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// One benchmark point. This is metadata plus a sourced value, not a solver result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    pub record_id: String,
    pub evidence_class: EvidenceClass,
    /// Required with an exact version/revision when evidence_class is ModelOutput.
    pub producer_model: Option<String>,
    /// Model release, source revision, or immutable build identifier.
    pub producer_revision: Option<String>,
    pub experiment_class: ExperimentClass,
    pub source: SourceReference,
    pub host_material: String,
    pub isotope_pair: String,
    pub loading_state: LoadingState,
    pub detection_channel: DetectionChannel,
    /// Required for beam-driven target experiments. The frame is carried by the measurement.
    pub incident_energy: Option<Measurement>,
    pub observable: Measurement,
    pub notes: Option<String>,
}

impl BenchmarkRecord {
    /// Validate all source, experiment-class and measurement requirements.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.record_id.trim().is_empty() {
            errors.push("record_id must not be empty".to_string());
        }
        if self.host_material.trim().is_empty() {
            errors.push("host_material must not be empty".to_string());
        }
        if self.isotope_pair.trim().is_empty() {
            errors.push("isotope_pair must not be empty".to_string());
        }

        match (
            self.evidence_class,
            self.producer_model.as_deref(),
            self.producer_revision.as_deref(),
        ) {
            (EvidenceClass::ModelOutput, Some(model), Some(revision))
                if !model.trim().is_empty() && !revision.trim().is_empty() => {}
            (EvidenceClass::ModelOutput, _, _) => errors.push(
                "model outputs require non-empty producer_model and producer_revision".to_string(),
            ),
            (_, None, None) => {}
            (_, _, _) => errors.push(
                "producer model metadata is only valid for ModelOutput evidence".to_string(),
            ),
        }

        let detector_channel_valid = match self.observable.quantity {
            QuantityKind::CalorimetricPower => {
                self.detection_channel == DetectionChannel::Calorimetry
            }
            QuantityKind::IncidentEnergy
            | QuantityKind::RelativeYield
            | QuantityKind::HydrideToMetalYieldRatio
            | QuantityKind::AbsoluteYield
            | QuantityKind::DetectorCountRate
            | QuantityKind::ArealDetectorCountRate
            | QuantityKind::TotalReactionRate
            | QuantityKind::VolumetricReactionRate
            | QuantityKind::EnhancementRatio => {
                self.detection_channel != DetectionChannel::Calorimetry
            }
            QuantityKind::LoadingRatio => {
                self.detection_channel != DetectionChannel::Calorimetry
            }
        };
        if !detector_channel_valid {
            errors.push(format!(
                "detection channel {:?} is incompatible with observable {:?}",
                self.detection_channel, self.observable.quantity
            ));
        }

        if let Err(source_errors) = self.source.validate(self.evidence_class) {
            errors.extend(source_errors);
        }
        if let Err(measurement_errors) = self.observable.validate() {
            errors.extend(
                measurement_errors
                    .into_iter()
                    .map(|error| format!("observable: {error}")),
            );
        }

        match self.experiment_class {
            ExperimentClass::BeamDrivenMetalTarget => {
                match &self.incident_energy {
                    Some(energy) => {
                        if energy.quantity != QuantityKind::IncidentEnergy {
                            errors.push(
                                "incident_energy field must contain an IncidentEnergy quantity"
                                    .to_string(),
                            );
                        }
                        if let Err(energy_errors) = energy.validate() {
                            errors.extend(
                                energy_errors
                                    .into_iter()
                                    .map(|error| format!("incident_energy: {error}")),
                            );
                        }
                    }
                    None => errors.push(
                        "beam-driven metal-target records require incident energy".to_string(),
                    ),
                }
                if self.loading_state == LoadingState::Unknown {
                    errors.push(
                        "beam-driven metal-target records require a declared loading state"
                            .to_string(),
                    );
                }
            }
            ExperimentClass::ElectrochemicalLoadingControl => {
                if self.loading_state != LoadingState::ElectrochemicallyLoaded {
                    errors.push(
                        "electrochemical loading controls require ElectrochemicallyLoaded state"
                            .to_string(),
                    );
                }
                if self.incident_energy.is_some() {
                    errors.push(
                        "electrochemical loading-only controls must not carry incident beam energy"
                            .to_string(),
                    );
                }
            }
            ExperimentClass::XrayTriggeredLattice | ExperimentClass::Other => {
                if let Some(energy) = &self.incident_energy {
                    if energy.quantity != QuantityKind::IncidentEnergy {
                        errors.push(
                            "incident_energy field must contain an IncidentEnergy quantity"
                                .to_string(),
                        );
                    }
                    if let Err(energy_errors) = energy.validate() {
                        errors.extend(
                            energy_errors
                                .into_iter()
                                .map(|error| format!("incident_energy: {error}")),
                        );
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(evidence_class: EvidenceClass) -> SourceReference {
        SourceReference {
            citation: "Synthetic benchmark fixture".to_string(),
            doi: None,
            stable_url: None,
            source_locator: "fixture:relative-yield-point".to_string(),
            content_sha256: if evidence_class == EvidenceClass::RawDataset {
                Some("a".repeat(64))
            } else {
                None
            },
            rights: DataRights::PaperOnly,
        }
    }

    fn relative_yield() -> Measurement {
        Measurement {
            quantity: QuantityKind::RelativeYield,
            value: 0.75,
            unit: MeasurementUnit::Dimensionless,
            basis: ValueBasis::NormalizedTo {
                reference_id: "reference-energy-point".to_string(),
            },
            uncertainty_status: UncertaintyStatus::Reported,
            uncertainty: Some(MeasurementUncertainty {
                magnitude: 0.05,
                kind: UncertaintyKind::StandardUncertainty,
                coverage_probability: None,
            }),
            energy_frame: None,
        }
    }

    fn incident_energy(frame: Option<EnergyFrame>) -> Measurement {
        Measurement {
            quantity: QuantityKind::IncidentEnergy,
            value: 1.0,
            unit: MeasurementUnit::KiloElectronVolt,
            basis: ValueBasis::Absolute,
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: frame,
        }
    }

    fn beam_record() -> BenchmarkRecord {
        BenchmarkRecord {
            record_id: "fixture-pdd-relative-yield-001".to_string(),
            evidence_class: EvidenceClass::SyntheticFixture,
            producer_model: None,
            producer_revision: None,
            experiment_class: ExperimentClass::BeamDrivenMetalTarget,
            source: source(EvidenceClass::SyntheticFixture),
            host_material: "palladium hydride".to_string(),
            isotope_pair: "D-D".to_string(),
            loading_state: LoadingState::ElectrochemicallyLoaded,
            detection_channel: DetectionChannel::Proton,
            incident_energy: Some(incident_energy(Some(EnergyFrame::CenterOfMass))),
            observable: relative_yield(),
            notes: Some("Synthetic schema fixture; not published experimental data.".to_string()),
        }
    }

    #[test]
    fn valid_beam_driven_record_passes() {
        assert!(beam_record().validate().is_ok());
    }

    #[test]
    fn beam_driven_record_requires_energy() {
        let mut record = beam_record();
        record.incident_energy = None;
        assert!(record.validate().is_err());
    }

    #[test]
    fn energy_requires_explicit_reference_frame() {
        let mut record = beam_record();
        record.incident_energy = Some(incident_energy(None));
        let errors = record.validate().unwrap_err();
        assert!(errors.iter().any(|error| error.contains("reference frame")));
    }

    #[test]
    fn relative_yield_must_be_dimensionless_and_normalized() {
        let mut measurement = relative_yield();
        measurement.unit = MeasurementUnit::Watt;
        assert!(measurement.validate().is_err());

        measurement.unit = MeasurementUnit::Dimensionless;
        measurement.basis = ValueBasis::Absolute;
        assert!(measurement.validate().is_err());
    }

    #[test]
    fn hydride_to_metal_ratio_requires_a_named_control() {
        let ratio = Measurement {
            quantity: QuantityKind::HydrideToMetalYieldRatio,
            value: 1.15,
            unit: MeasurementUnit::Dimensionless,
            basis: ValueBasis::RatioToControl {
                control_id: "beam-only-metal-control".to_string(),
            },
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(ratio.validate().is_ok());

        let mut unnamed = ratio;
        unnamed.basis = ValueBasis::RatioToControl {
            control_id: " ".to_string(),
        };
        assert!(unnamed.validate().is_err());
    }

    #[test]
    fn areal_and_total_detector_rates_cannot_be_conflated() {
        let total = Measurement {
            quantity: QuantityKind::DetectorCountRate,
            value: 10.0,
            unit: MeasurementUnit::CountsPerSecond,
            basis: ValueBasis::Absolute,
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(total.validate().is_ok());

        let mut mislabeled = total;
        mislabeled.unit = MeasurementUnit::CountsPerSecondPerSquareCentimeter;
        assert!(mislabeled.validate().is_err());

        let areal = Measurement {
            quantity: QuantityKind::ArealDetectorCountRate,
            value: 10.0,
            unit: MeasurementUnit::CountsPerSecondPerSquareCentimeter,
            basis: ValueBasis::Absolute,
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(areal.validate().is_ok());
    }

    #[test]
    fn volumetric_and_total_reaction_rates_have_distinct_units() {
        let volumetric = Measurement {
            quantity: QuantityKind::VolumetricReactionRate,
            value: 1.0e6,
            unit: MeasurementUnit::ReactionsPerSecondPerCubicCentimeter,
            basis: ValueBasis::Absolute,
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(volumetric.validate().is_ok());

        let mut mislabeled = volumetric;
        mislabeled.unit = MeasurementUnit::ReactionsPerSecond;
        assert!(mislabeled.validate().is_err());
    }

    #[test]
    fn calorimetric_power_requires_calorimetry_channel() {
        let power = Measurement {
            quantity: QuantityKind::CalorimetricPower,
            value: 0.2,
            unit: MeasurementUnit::Watt,
            basis: ValueBasis::Absolute,
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(power.validate().is_ok());

        let mut record = beam_record();
        record.observable = power;
        assert!(record.validate().is_err());

        record.detection_channel = DetectionChannel::Calorimetry;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn model_output_requires_model_and_revision_identity() {
        let mut record = beam_record();
        record.evidence_class = EvidenceClass::ModelOutput;
        assert!(record.validate().is_err());

        record.producer_model = Some("example-screening-model".to_string());
        record.producer_revision = Some("git:0123456789abcdef".to_string());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn model_enhancement_is_not_a_power_measurement() {
        let enhancement = Measurement {
            quantity: QuantityKind::EnhancementRatio,
            value: 1.0e18,
            unit: MeasurementUnit::Dimensionless,
            basis: ValueBasis::EnhancementVsModel {
                model_id: "bare-nucleus-reference".to_string(),
            },
            uncertainty_status: UncertaintyStatus::NotReported,
            uncertainty: None,
            energy_frame: None,
        };
        assert!(enhancement.validate().is_ok());

        let mut mislabeled = enhancement;
        mislabeled.unit = MeasurementUnit::Watt;
        assert!(mislabeled.validate().is_err());
    }

    #[test]
    fn raw_dataset_requires_content_digest() {
        let mut record = beam_record();
        record.evidence_class = EvidenceClass::RawDataset;
        record.source = source(EvidenceClass::RawDataset);
        assert!(record.validate().is_ok());

        record.source.content_sha256 = None;
        assert!(record.validate().is_err());
    }

    #[test]
    fn digitized_figure_requires_a_content_digest() {
        let mut record = beam_record();
        record.evidence_class = EvidenceClass::DigitizedFigure;
        record.source = source(EvidenceClass::DigitizedFigure);
        assert!(record.validate().is_err());

        record.source.content_sha256 = Some("b".repeat(64));
        assert!(record.validate().is_ok());
    }

    #[test]
    fn malformed_content_digest_is_rejected() {
        let mut record = beam_record();
        record.evidence_class = EvidenceClass::RawDataset;
        record.source = source(EvidenceClass::RawDataset);
        record.source.content_sha256 = Some("z".repeat(64));
        assert!(record.validate().is_err());
    }

    #[test]
    fn electrochemical_control_cannot_be_relabelled_as_beam_driven() {
        let mut record = beam_record();
        record.experiment_class = ExperimentClass::ElectrochemicalLoadingControl;
        assert!(record.validate().is_err());

        record.incident_energy = None;
        record.loading_state = LoadingState::ElectrochemicallyLoaded;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let mut measurement = relative_yield();
        measurement.value = f64::NAN;
        assert!(measurement.validate().is_err());
    }

    #[test]
    fn uncertainty_status_must_match_payload() {
        let mut measurement = relative_yield();
        measurement.uncertainty = None;
        assert!(measurement.validate().is_err());

        measurement.uncertainty_status = UncertaintyStatus::NotReported;
        assert!(measurement.validate().is_ok());
    }

    #[test]
    fn expanded_uncertainty_requires_coverage_probability() {
        let mut measurement = relative_yield();
        measurement.uncertainty = Some(MeasurementUncertainty {
            magnitude: 0.05,
            kind: UncertaintyKind::ExpandedUncertainty,
            coverage_probability: None,
        });
        assert!(measurement.validate().is_err());
    }
}
