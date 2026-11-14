// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
//! # Spark Engine: Lattice Confinement Fusion Physics
//!
//! A physics-honest simulation framework for LCF research, focused on:
//! - Bosch-Hale branch-resolved bare D-D reference fits plus an exploratory constant-S
//!   Gamow kernel for screening/phonon sensitivity studies
//! - Q factor accounting with explicit model and system-boundary limitations
//! - Lattice lifetime modeling with DPA tracking
//! - Anomaly investigation (Path C): Understanding observed vs predicted rates
//! - Neutron source design (Path D): Compact neutron generator applications
//!
//! ## Numerical baseline qualification
//!
//! The bare-nucleus reference is branch-resolved Bosch-Hale cross-sections
//! and Maxwellian reactivities in bosch_hale.rs, with numerical integration
//! cross-checks and explicit published validity domains. The legacy constant-S
//! Gamow kernel in physics.rs remains an exploratory screening/phonon sensitivity
//! model; its 1–20 keV NRL comparison has a 35% approximation envelope. The
//! 300 K / beam-lattice regime is outside both thermonuclear fit domains: results
//! there are extrapolations, not validated reactivities or measured LCF rates.
//!
//! NASA Glenn reported detector-efficiency-corrected fusion-energy neutron
//! rates scaled to the sample location of 1.5 ± 0.3 × 10³ and
//! 1.6 ± 0.3 × 10³ neutrons/s for two samples. These are inferred source
//! rates, not raw detector count rates (Steinetz et al., Phys. Rev. C 101,
//! 044610 (2020), DOI 10.1103/PhysRevC.101.044610).
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
//!
//! ## Usage
//!
//! ```rust
//! use spark_engine::physics::{GamowIntegration, QFactor, QFactorParams};
//! use spark_engine::anomaly::AnomalyAnalysis;
//!
//! // Model-screening example only; 300 K is outside the independent
//! // 1–20 keV reactivity regression domain.
//! let gamow = GamowIntegration::dd_rate(300.0, 309.0, 2);
//! let (q, achievable) = QFactor::compute(&gamow, &QFactorParams::lcf_typical());
//! assert!(q < 1.0, "Q remains below unity in this modelled scenario");
//!
//! // API illustration with synthetic, boundary-matched rates only.
//! // Do not substitute a published source-location rate until its sample
//! // geometry, detector correction, energy distribution and model domain match.
//! let synthetic_observed_source_rate = 100.0;
//! let synthetic_predicted_source_rate = 1.0;
//! let gap = AnomalyAnalysis::compute_gap(
//!     synthetic_observed_source_rate,
//!     synthetic_predicted_source_rate,
//! );
//! ```

// Core physics modules
pub mod anomaly;
pub mod bosch_hale;
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
pub use shielding::{NeutronShielding, ShieldingResult};

// Path C re-exports
pub use experimental_design::{ExperimentDesign, ExperimentDesigner, ExperimentalProgram};
pub use hypothesis_models::{
    HotSpotModel, HypothesisComparison, HypothesisType, PhononCascadeModel, SuperScreeningModel,
};
pub use literature::{
    Credibility, HostMaterial as LitHostMaterial, LiteratureDatabase, Observation, TriggerMethod,
};
pub use rate_gap::{ExperimentalConditions, RateGapAnalysis, RateGapCalculator};

// Bosch-Hale bare-nucleus reference models (kept separate from lattice hypotheses).
pub use bosch_hale::{
    BoschHaleError, DDBoschHaleReactivity, DDBranch, IntegratedReactivity,
    dd_branch_maxwellian_cm3_s, dd_cross_section_mb, dd_maxwellian_cm3_s,
    integrate_dd_branch_maxwellian, INTEGRATION_MIN_TEMPERATURE_KEV,
};

/// Spark Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Physics constants used throughout
pub mod constants {
    /// Low-energy D-D neutron-branch S-factor at E→0 (keV·barn), Bosch-Hale.
    /// Value is 53,701 millibarn·keV converted to 53.701 barn·keV.
    /// Reference implementation coefficients: WarpX BoschHaleFusionCrossSection.
    pub const DD_S_FACTOR_NEUTRON: f64 = 53.701;
    /// Low-energy D-D proton-branch S-factor at E→0 (keV·barn), Bosch-Hale.
    /// Value is 55,576 millibarn·keV converted to 55.576 barn·keV.
    /// Reference implementation coefficients: WarpX BoschHaleFusionCrossSection.
    pub const DD_S_FACTOR_PROTON: f64 = 55.576;
    /// Approximate total D-D S-factor (both branches; keV·barn).
    ///
    /// The constant-S integration is a screening baseline/approximation, not a
    /// substitute for the energy-dependent Bosch-Hale branch fits.
    pub const DD_S_FACTOR: f64 = DD_S_FACTOR_NEUTRON + DD_S_FACTOR_PROTON;

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

    /// NASA Steinetz TS1575 detector-efficiency-corrected source rate at sample location (n/s).
    /// Source: Steinetz et al., Phys. Rev. C 101, 044610 (2020), Table/Results text.
    pub const NASA_STEINETZ_TS1575_RATE_S: f64 = 1.5e3;
    /// Reported standard uncertainty for the TS1575 source rate (n/s).
    pub const NASA_STEINETZ_TS1575_RATE_UNCERTAINTY_S: f64 = 0.3e3;
    /// NASA Steinetz TS1576 detector-efficiency-corrected source rate at sample location (n/s).
    pub const NASA_STEINETZ_TS1576_RATE_S: f64 = 1.6e3;
    /// Reported standard uncertainty for the TS1576 source rate (n/s).
    pub const NASA_STEINETZ_TS1576_RATE_UNCERTAINTY_S: f64 = 0.3e3;
    /// Legacy rounded sample-location source-rate placeholder (n/s).
    /// Not an areal detector count rate; prefer the sample-specific constants above.
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
