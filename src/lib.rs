// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
//! # Spark Engine: Lattice Confinement Fusion Physics
//!
//! A physics-honest simulation framework for LCF research, focused on:
//! - Rigorous Gamow integration with screening and phonon effects
//! - Q factor calculation with honest assessment (Q << 1 at room temperature)
//! - Lattice lifetime modeling with DPA tracking
//! - Anomaly investigation (Path C): Understanding observed vs predicted rates
//! - Neutron source design (Path D): Compact neutron generator applications
//!
//! ## Key Finding
//!
//! Standard Gamow physics predicts `<σv>` ~ 10⁻⁵⁰ cm³/s at room temperature,
//! yet NASA observed ~10³ neutrons/s. This gap is the central mystery.
//!
//! ## Modules
//!
//! - **physics**: Core Gamow integration, Q factor, screening calculations
//! - **lattice**: DPA tracking, lattice lifetime, D/Pd ratio decay
//! - **anomaly**: Gap analysis, hypothesis framework
//! - **literature**: Database of LCF/LENR observations with credibility scoring
//! - **hypothesis_models**: Detailed physics models (hot spots, phonon cascade, screening)
//! - **rate_gap**: Quantitative rate gap calculator with sensitivity analysis
//! - **experimental_design**: Tools for designing discriminating experiments
//! - **neutron_source**: Compact neutron generator design
//! - **shielding**: Neutron shielding calculations
//! - **bayesian**: Probability distribution over anomaly hypotheses, updated
//!   from experimental outcomes
//! - **optimal_experiment**: Expected-information-gain ranking and
//!   budget-constrained experiment sequencing
//! - **qecon**: Surface-normalized economic gain and design thresholds
//!
//! ## Usage
//!
//! ```rust
//! use spark_engine::physics::{GamowIntegration, QFactor, QFactorParams};
//! use spark_engine::anomaly::AnomalyAnalysis;
//!
//! // Standard physics calculation
//! let gamow = GamowIntegration::dd_rate(300.0, 309.0, 2);
//! let (q, achievable) = QFactor::compute(&gamow, &QFactorParams::lcf_typical());
//! assert!(q < 1.0, "Q < 1 at room temperature - physics is honest");
//!
//! // Anomaly investigation
//! let observed = 1e3; // NASA neutron rate
//! let n_d = 4.8e22; // D atoms/cm³
//! let volume = 0.01; // cm³
//! let predicted = gamow.to_neutron_rate(n_d, volume);
//! let gap = AnomalyAnalysis::compute_gap(observed, predicted);
//! ```

// Core physics modules
pub mod anomaly;
pub mod error;
pub mod lattice;
pub mod neutron_source;
pub mod physics;
pub mod shielding;

// Path C: Anomaly Investigation modules
pub mod experimental_design;
pub mod hypothesis_models;
pub mod literature;
pub mod rate_gap;
pub mod uncertainty;

// Integration modules
pub mod bridge;

// Explicit plant power boundary and first-order economics
pub mod plant_energy_balance;

// Q_econ surface-normalized fusion-plant economics
pub mod qecon;

// Evidence-aware comparison of Q_econ thresholds with declared engineering power-density limits
pub mod qecon_feasibility;

// Conservation-ledger mapping from Q_econ extraction power to divertor target loads
pub mod power_exhaust_bridge;

// Sensitivity, scenario sweeps and correlated uncertainty for Q_econ
pub mod qecon_exploration;

// First-order reliability/availability screening (not full RAMI)
pub mod plant_availability;

// Evidence-aware sub-keV metallic hydride yield models
pub mod sub_kev_yield;

// Extended fusion physics (D-T, p-B11, muon-catalyzed)
pub mod reactions;

// Multi-physics discovery engine
pub mod multi_physics;

// CfC temporal dynamics analysis
pub mod cfc_physics;

// Bayesian hypothesis discrimination
pub mod bayesian;

// Bayesian optimal experiment design
pub mod optimal_experiment;

// Core re-exports
pub use anomaly::{AnomalyAnalysis, AnomalyHypothesis};
pub use bayesian::{ALL_HYPOTHESES, HypothesisBelief, ObservedOutcome, OutcomeClasses};
pub use error::{Result, SparkError};
pub use lattice::{LatticeLifetime, LatticeState};
pub use neutron_source::{NeutronSourceDesign, SourceSpec};
pub use optimal_experiment::{
    ExperimentInfoGain, SequenceStep, expected_information_gain, generate_proposal_markdown,
    greedy_sequence, rank_experiments, standard_candidate_experiments,
};
pub use physics::{GamowIntegration, GamowResult, QFactor, QFactorParams};
pub use plant_availability::{
    AVAILABILITY_MODEL_REVISION, AvailabilityGroup, AvailabilityGroupResult,
    ComponentAvailabilityResult, ComponentRepairProfile, PlantAvailabilityInput,
    PlantAvailabilityResult, calculate_plant_availability,
};

pub use plant_energy_balance::{
    PlantEconomicsInput, PlantEconomicsResult, PlantPowerBalance, PlantPowerBalanceInput,
    calculate_lcoe, calculate_plant_power_balance,
};
pub use qecon_feasibility::{
    QECON_FEASIBILITY_REVISION, QeconEnvelopeStatus, QeconFeasibilityResult,
    QeconFeasibilityScenario, QeconPowerDensityEnvelope, evaluate_qecon_power_density_envelope,
};
pub use power_exhaust_bridge::{
    POWER_EXHAUST_BRIDGE_REVISION, DivertorTargetInput, DivertorTargetLoad,
    ExhaustPowerBalance, PowerExhaustBridgeInput, PowerExhaustBridgeResult, PowerSurfaceArea,
    PowerSurfaceRole, PowerTimeBasis, PowerTimeBasisKind, map_qecon_to_divertor_loads,
};
pub use qecon::{
    QECON_MODEL_REVISION, QeconAsymptoticLimits, QeconDesignParameters, QeconEvidenceClass,
    QeconInput, QeconResult, QeconScenario, QeconScenarioResult, calculate_qecon,
    calculate_qecon_asymptotic_limits, calculate_qecon_scenario,
    net_electric_efficiency_from_power_balance, required_fusion_power_density_for_qecon,
};
pub use qecon_exploration::{
    DifferenceMethod, MAX_QECON_UNCERTAINTY_SAMPLES, QECON_EXPLORATION_REVISION,
    QECON_PARAMETER_COUNT, QeconGlobalAssociation, QeconParameter, QeconProbabilityEstimate,
    QeconScalarStatistics, QeconSensitivityEstimate, QeconSobolIndex, QeconSobolInput,
    QeconSobolResult, QeconTriangularMarginal, QeconUncertaintyInput, QeconUncertaintyResult,
    QeconUncertaintySample, QeconUncertaintyScenario, QeconUncertaintyScenarioResult,
    estimate_qecon_sobol_indices, propagate_qecon_scenario_uncertainty,
    propagate_qecon_uncertainty, rank_qecon_global_associations, rank_qecon_sensitivities,
};
pub use shielding::{NeutronShielding, ShieldingResult};
pub use sub_kev_yield::{
    DatasetAvailability, DetectionChannel, HomogeneousScreeningFit, LoadingCondition,
    RelativeYieldPoint, RelativeYieldSeries, ScreeningParameterStatus, SubKevHostMaterial,
    YieldDataOrigin, YieldProvenance, alr_screening_enhancement,
    equivalent_homogeneous_screening_kev, published_high_energy_fits, two_environment_enhancement,
};

// Path C re-exports
pub use experimental_design::{ExperimentDesign, ExperimentDesigner, ExperimentalProgram};
pub use hypothesis_models::{
    HotSpotModel, HypothesisComparison, HypothesisType, PhononCascadeModel, SuperScreeningModel,
};
pub use literature::{
    Credibility, HostMaterial as LitHostMaterial, LiteratureDatabase, Observation, TriggerMethod,
};
pub use rate_gap::{ExperimentalConditions, RateGapAnalysis, RateGapCalculator};

/// Spark Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Physics constants used throughout
pub mod constants {
    /// D-D astrophysical S-factor (keV·barn)
    pub const DD_S_FACTOR: f64 = 55.0;

    /// D-D Gamow constant (keV^0.5)
    pub const DD_GAMOW_CONSTANT: f64 = 31.38;

    /// D-D Q-value, neutron branch (MeV)
    pub const DD_Q_NEUTRON: f64 = 3.269;

    /// D-D Q-value, proton branch (MeV)
    pub const DD_Q_PROTON: f64 = 4.033;

    /// D-D average Q-value (MeV)
    pub const DD_Q_AVERAGE: f64 = 3.651;

    /// D-D neutron energy (MeV)
    pub const DD_NEUTRON_ENERGY: f64 = 2.45;

    /// D-T neutron energy (MeV)
    pub const DT_NEUTRON_ENERGY: f64 = 14.1;

    /// Electron screening in Pd (eV) - Raiola et al. 2004
    pub const SCREENING_PD_EV: f64 = 309.0;

    /// PdD optical phonon energy (keV)
    pub const PDD_PHONON_KEV: f64 = 0.056;

    /// Boltzmann constant (keV/K)
    pub const KB_KEV: f64 = 8.617e-8;

    /// NASA observed neutron rate (n/s per cm²)
    pub const NASA_OBSERVED_RATE: f64 = 1e3;

    /// Pd density (g/cm³)
    pub const PD_DENSITY: f64 = 12.02;

    /// Pd atomic mass (u)
    pub const PD_ATOMIC_MASS: f64 = 106.42;
}

/// Prelude for common imports
pub mod prelude {
    // Core physics
    pub use crate::anomaly::{AnomalyAnalysis, AnomalyGap, AnomalyHypothesis};
    pub use crate::lattice::{LatticeLifetime, LatticeState};
    pub use crate::neutron_source::{NeutronSourceDesign, NeutronSourceDesigner, SourceSpec};
    pub use crate::physics::{GamowIntegration, GamowResult, QFactor, QFactorParams};
    pub use crate::plant_availability::{
        AvailabilityGroup, ComponentRepairProfile, PlantAvailabilityInput, PlantAvailabilityResult,
        calculate_plant_availability,
    };

    pub use crate::plant_energy_balance::{
        PlantEconomicsInput, PlantEconomicsResult, PlantPowerBalance, PlantPowerBalanceInput,
        calculate_lcoe, calculate_plant_power_balance,
    };
    pub use crate::qecon::{
        QeconDesignParameters, QeconEvidenceClass, QeconInput, QeconResult, QeconScenario,
        calculate_qecon, calculate_qecon_scenario, net_electric_efficiency_from_power_balance,
        required_fusion_power_density_for_qecon,
    };
    pub use crate::shielding::{NeutronShielding, ShieldingResult};

    // Path C: Anomaly investigation
    pub use crate::experimental_design::{
        ExperimentDesign, ExperimentDesigner, ExperimentalProgram,
    };
    pub use crate::hypothesis_models::{
        HotSpotModel, HypothesisComparison, HypothesisType, PhononCascadeModel, SuperScreeningModel,
    };
    pub use crate::literature::{LiteratureDatabase, Observation};
    pub use crate::rate_gap::{RateGapAnalysis, RateGapCalculator};

    pub use crate::constants;
    pub use crate::{Result, SparkError};
}
