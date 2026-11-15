//! Independent analytical oracle for fusion event-rate and power stoichiometry.
//!
//! The crate deliberately does not depend on or call Spark production code.
//! It checks algebraic channel accounting only; it is not a plasma solver,
//! reactivity model, physical validation or reactor model.

/// Exact conversion from MeV to joules.
pub const MEV_TO_JOULE: f64 = 1.602_176_634e-13;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Species {
    Deuterium,
    Tritium,
    Helium3,
    Proton,
    Boron11,
    Lithium6,
    Lithium7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReactionChannel {
    DdNeutron,
    DdProton,
    Dt,
    DHe3,
    PB11,
    PLi6,
    PLi7,
    He3He3,
}

impl ReactionChannel {
    /// Ordered reactants. Order is explicit so input species cannot be swapped silently.
    pub const fn reactants(self) -> (Species, Species) {
        match self {
            Self::DdNeutron | Self::DdProton => (Species::Deuterium, Species::Deuterium),
            Self::Dt => (Species::Deuterium, Species::Tritium),
            Self::DHe3 => (Species::Deuterium, Species::Helium3),
            Self::PB11 => (Species::Proton, Species::Boron11),
            Self::PLi6 => (Species::Proton, Species::Lithium6),
            Self::PLi7 => (Species::Proton, Species::Lithium7),
            Self::He3He3 => (Species::Helium3, Species::Helium3),
        }
    }

    pub const fn is_identical_reactants(self) -> bool {
        let (first, second) = self.reactants();
        matches!(
            (first, second),
            (Species::Deuterium, Species::Deuterium) | (Species::Helium3, Species::Helium3)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberDensityCm3(f64);

impl NumberDensityCm3 {
    pub fn new(value: f64) -> Result<Self, InputError> {
        if !value.is_finite() {
            return Err(InputError::NonFiniteDensity);
        }
        if value <= 0.0 {
            return Err(InputError::NonPositiveDensity);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactivityCm3S(f64);

impl ReactivityCm3S {
    pub fn new(value: f64) -> Result<Self, InputError> {
        if !value.is_finite() {
            return Err(InputError::NonFiniteReactivity);
        }
        if value <= 0.0 {
            return Err(InputError::NonPositiveReactivity);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyMeV(f64);

impl EnergyMeV {
    pub fn new(value: f64) -> Result<Self, InputError> {
        if !value.is_finite() {
            return Err(InputError::NonFiniteEnergy);
        }
        if value <= 0.0 {
            return Err(InputError::NonPositiveEnergy);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReactantDensities {
    /// One density is supplied for a pair of identical reactants.
    Identical {
        species: Species,
        density: NumberDensityCm3,
    },
    /// Both unlike-reactant species and densities are named explicitly.
    Distinct {
        first_species: Species,
        first_density: NumberDensityCm3,
        second_species: Species,
        second_density: NumberDensityCm3,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    NonFiniteDensity,
    NonPositiveDensity,
    NonFiniteReactivity,
    NonPositiveReactivity,
    NonFiniteEnergy,
    NonPositiveEnergy,
    ExpectedIdenticalReactants,
    ExpectedDistinctReactants,
    FirstSpeciesMismatch,
    SecondSpeciesMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventRateCm3S(f64);

impl EventRateCm3S {
    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerDensityWcm3(f64);

impl PowerDensityWcm3 {
    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelAccounting {
    pub channel: ReactionChannel,
    pub event_rate: EventRateCm3S,
    pub power_density: PowerDensityWcm3,
    /// 2 for identical reactants, 1 for unlike reactants.
    pub symmetry_denominator: u8,
    pub q_value: EnergyMeV,
    pub reactivity: ReactivityCm3S,
}

/// Compute reaction events and reaction power for one explicitly named channel.
pub fn account_channel(
    channel: ReactionChannel,
    densities: ReactantDensities,
    reactivity: ReactivityCm3S,
    q_value: EnergyMeV,
) -> Result<ChannelAccounting, InputError> {
    let (expected_first, expected_second) = channel.reactants();
    let (density_product, symmetry_denominator) = match densities {
        ReactantDensities::Identical { species, density } => {
            if !channel.is_identical_reactants() {
                return Err(InputError::ExpectedDistinctReactants);
            }
            if species != expected_first || species != expected_second {
                return Err(InputError::FirstSpeciesMismatch);
            }
            (density.value() * density.value(), 2)
        }
        ReactantDensities::Distinct {
            first_species,
            first_density,
            second_species,
            second_density,
        } => {
            if channel.is_identical_reactants() {
                return Err(InputError::ExpectedIdenticalReactants);
            }
            if first_species != expected_first {
                return Err(InputError::FirstSpeciesMismatch);
            }
            if second_species != expected_second {
                return Err(InputError::SecondSpeciesMismatch);
            }
            (first_density.value() * second_density.value(), 1)
        }
    };

    let event_rate = density_product * reactivity.value() / f64::from(symmetry_denominator);
    let power_density = event_rate * q_value.value() * MEV_TO_JOULE;

    if !event_rate.is_finite() || !power_density.is_finite() {
        return Err(InputError::NonFiniteEnergy);
    }

    Ok(ChannelAccounting {
        channel,
        event_rate: EventRateCm3S(event_rate),
        power_density: PowerDensityWcm3(power_density),
        symmetry_denominator,
        q_value,
        reactivity,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DDBranchInputs {
    pub neutron_reactivity: ReactivityCm3S,
    pub neutron_q: EnergyMeV,
    pub proton_reactivity: ReactivityCm3S,
    pub proton_q: EnergyMeV,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DdTotalAccounting {
    pub neutron_branch: ChannelAccounting,
    pub proton_branch: ChannelAccounting,
    pub total_event_rate: EventRateCm3S,
    pub total_power_density: PowerDensityWcm3,
}

/// Account for both D-D branches independently, then sum them exactly once.
pub fn account_dd_total(
    density: NumberDensityCm3,
    branches: DDBranchInputs,
) -> Result<DdTotalAccounting, InputError> {
    let densities = ReactantDensities::Identical {
        species: Species::Deuterium,
        density,
    };
    let neutron_branch = account_channel(
        ReactionChannel::DdNeutron,
        densities,
        branches.neutron_reactivity,
        branches.neutron_q,
    )?;
    let proton_branch = account_channel(
        ReactionChannel::DdProton,
        densities,
        branches.proton_reactivity,
        branches.proton_q,
    )?;

    Ok(DdTotalAccounting {
        neutron_branch,
        proton_branch,
        total_event_rate: EventRateCm3S(
            neutron_branch.event_rate.value() + proton_branch.event_rate.value(),
        ),
        total_power_density: PowerDensityWcm3(
            neutron_branch.power_density.value() + proton_branch.power_density.value(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn density(value: f64) -> Result<NumberDensityCm3, InputError> {
        NumberDensityCm3::new(value)
    }

    fn sv(value: f64) -> Result<ReactivityCm3S, InputError> {
        ReactivityCm3S::new(value)
    }

    fn q(value: f64) -> Result<EnergyMeV, InputError> {
        EnergyMeV::new(value)
    }

    fn identical_d(channel_species: Species, n: f64) -> ReactantDensities {
        ReactantDensities::Identical {
            species: channel_species,
            density: density(n).unwrap(),
        }
    }

    fn distinct(first: Species, n_first: f64, second: Species, n_second: f64) -> ReactantDensities {
        ReactantDensities::Distinct {
            first_species: first,
            first_density: density(n_first).unwrap(),
            second_species: second,
            second_density: density(n_second).unwrap(),
        }
    }

    fn dd_inputs() -> DDBranchInputs {
        DDBranchInputs {
            neutron_reactivity: sv(2.0e-24).unwrap(),
            neutron_q: q(3.269).unwrap(),
            proton_reactivity: sv(3.0e-24).unwrap(),
            proton_q: q(4.033).unwrap(),
        }
    }

    #[test]
    fn dd_neutron_branch_known_answer() {
        let result = account_channel(
            ReactionChannel::DdNeutron,
            identical_d(Species::Deuterium, 2.0e20),
            sv(2.0e-24).unwrap(),
            q(3.269).unwrap(),
        )
        .unwrap();
        let expected_rate = (2.0e20_f64.powi(2) * 2.0e-24) / 2.0;
        let expected_power = expected_rate * 3.269 * MEV_TO_JOULE;
        assert_eq!(result.symmetry_denominator, 2);
        assert!((result.event_rate.value() - expected_rate).abs() <= expected_rate * 1.0e-14);
        assert!((result.power_density.value() - expected_power).abs() <= expected_power * 1.0e-14);
    }

    #[test]
    fn dd_proton_branch_known_answer() {
        let result = account_channel(
            ReactionChannel::DdProton,
            identical_d(Species::Deuterium, 2.0e20),
            sv(3.0e-24).unwrap(),
            q(4.033).unwrap(),
        )
        .unwrap();
        let expected_rate = (2.0e20_f64.powi(2) * 3.0e-24) / 2.0;
        let expected_power = expected_rate * 4.033 * MEV_TO_JOULE;
        assert_eq!(result.symmetry_denominator, 2);
        assert!((result.event_rate.value() - expected_rate).abs() <= expected_rate * 1.0e-14);
        assert!((result.power_density.value() - expected_power).abs() <= expected_power * 1.0e-14);
    }

    #[test]
    fn dd_total_is_sum_of_branch_event_rates_and_powers() {
        let n = density(2.0e20).unwrap();
        let result = account_dd_total(n, dd_inputs()).unwrap();
        let expected_rate = n.value().powi(2) * (2.0e-24 + 3.0e-24) / 2.0;
        let expected_power =
            n.value().powi(2) * (2.0e-24 * 3.269 + 3.0e-24 * 4.033) / 2.0 * MEV_TO_JOULE;
        assert!((result.total_event_rate.value() - expected_rate).abs() <= expected_rate * 1.0e-14);
        assert!(
            (result.total_power_density.value() - expected_power).abs() <= expected_power * 1.0e-14
        );
    }

    #[test]
    fn dt_unlike_reactants_have_no_symmetry_divisor() {
        let result = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Deuterium, 2.0e20, Species::Tritium, 3.0e20),
            sv(4.0e-22).unwrap(),
            q(17.6).unwrap(),
        )
        .unwrap();
        assert_eq!(result.symmetry_denominator, 1);
        assert_eq!(result.event_rate.value(), 2.0e20 * 3.0e20 * 4.0e-22);
    }

    #[test]
    fn dhe3_unlike_reactants_have_no_symmetry_divisor() {
        let result = account_channel(
            ReactionChannel::DHe3,
            distinct(Species::Deuterium, 2.0e20, Species::Helium3, 5.0e19),
            sv(6.0e-23).unwrap(),
            q(18.3).unwrap(),
        )
        .unwrap();
        assert_eq!(result.symmetry_denominator, 1);
        assert_eq!(result.event_rate.value(), 2.0e20 * 5.0e19 * 6.0e-23);
    }

    #[test]
    fn pb11_unlike_reactants_have_no_symmetry_divisor() {
        let result = account_channel(
            ReactionChannel::PB11,
            distinct(Species::Proton, 1.0e20, Species::Boron11, 1.0e19),
            sv(7.0e-25).unwrap(),
            q(8.7).unwrap(),
        )
        .unwrap();
        assert_eq!(result.symmetry_denominator, 1);
        assert_eq!(result.event_rate.value(), 1.0e20 * 1.0e19 * 7.0e-25);
    }

    #[test]
    fn he3he3_identical_reactants_have_one_half_divisor() {
        let result = account_channel(
            ReactionChannel::He3He3,
            identical_d(Species::Helium3, 4.0e19),
            sv(8.0e-24).unwrap(),
            q(12.9).unwrap(),
        )
        .unwrap();
        assert_eq!(result.symmetry_denominator, 2);
        assert_eq!(
            result.event_rate.value(),
            4.0e19_f64.powi(2) * 8.0e-24 / 2.0
        );
    }

    #[test]
    fn doubling_one_unlike_density_doubles_rate() {
        let base = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Deuterium, 1.0e20, Species::Tritium, 2.0e20),
            sv(5.0e-22).unwrap(),
            q(17.6).unwrap(),
        )
        .unwrap();
        let doubled = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Deuterium, 2.0e20, Species::Tritium, 2.0e20),
            sv(5.0e-22).unwrap(),
            q(17.6).unwrap(),
        )
        .unwrap();
        assert!((doubled.event_rate.value() / base.event_rate.value() - 2.0).abs() < 1.0e-14);
    }

    #[test]
    fn doubling_identical_density_quadruples_rate() {
        let base = account_channel(
            ReactionChannel::DdNeutron,
            identical_d(Species::Deuterium, 1.0e20),
            sv(2.0e-24).unwrap(),
            q(3.269).unwrap(),
        )
        .unwrap();
        let doubled = account_channel(
            ReactionChannel::DdNeutron,
            identical_d(Species::Deuterium, 2.0e20),
            sv(2.0e-24).unwrap(),
            q(3.269).unwrap(),
        )
        .unwrap();
        assert!((doubled.event_rate.value() / base.event_rate.value() - 4.0).abs() < 1.0e-14);
    }

    #[test]
    fn branch_specific_q_values_are_not_replaced_with_average() {
        let result = account_dd_total(density(1.0e20).unwrap(), dd_inputs()).unwrap();
        let correct = 1.0e40 * (2.0e-24 * 3.269 + 3.0e-24 * 4.033) / 2.0 * MEV_TO_JOULE;
        let incorrect_average = 1.0e40 * (2.0e-24 + 3.0e-24) / 2.0 * 3.651 * MEV_TO_JOULE;
        assert!((result.total_power_density.value() - correct).abs() <= correct * 1.0e-14);
        assert!((result.total_power_density.value() - incorrect_average).abs() > correct * 1.0e-3);
    }

    #[test]
    fn dd_branch_sum_is_not_multiplied_by_another_branch_fraction() {
        let result = account_dd_total(density(1.0e20).unwrap(), dd_inputs()).unwrap();
        let branch_sum =
            result.neutron_branch.event_rate.value() + result.proton_branch.event_rate.value();
        let incorrectly_halved = branch_sum * 0.5;
        assert!((result.total_event_rate.value() - branch_sum).abs() <= branch_sum * 1.0e-14);
        assert!((result.total_event_rate.value() - incorrectly_halved).abs() > branch_sum * 0.1);
    }

    #[test]
    fn universal_quarter_factor_is_not_the_oracle_formula() {
        let identical = account_channel(
            ReactionChannel::DdNeutron,
            identical_d(Species::Deuterium, 1.0e20),
            sv(2.0e-24).unwrap(),
            q(3.269).unwrap(),
        )
        .unwrap();
        let unlike = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Deuterium, 1.0e20, Species::Tritium, 1.0e20),
            sv(2.0e-24).unwrap(),
            q(17.6).unwrap(),
        )
        .unwrap();
        let expected_identical = 1.0e40 * 2.0e-24 / 2.0;
        let expected_unlike = 1.0e40 * 2.0e-24;
        assert_eq!(identical.event_rate.value(), expected_identical);
        assert_eq!(unlike.event_rate.value(), expected_unlike);
        assert_ne!(identical.symmetry_denominator, 4);
        assert_ne!(unlike.symmetry_denominator, 4);
    }

    #[test]
    fn meV_to_joule_conversion_has_exact_declared_constant() {
        assert_eq!(MEV_TO_JOULE, 1.602_176_634e-13);
        let result = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Deuterium, 1.0e20, Species::Tritium, 1.0e20),
            sv(1.0e-22).unwrap(),
            q(1.0).unwrap(),
        )
        .unwrap();
        assert_eq!(result.power_density.value(), 1.0e18 * MEV_TO_JOULE);
    }

    #[test]
    fn negative_density_is_rejected() {
        assert_eq!(density(-1.0), Err(InputError::NonPositiveDensity));
    }

    #[test]
    fn zero_density_is_rejected() {
        assert_eq!(density(0.0), Err(InputError::NonPositiveDensity));
    }

    #[test]
    fn nonfinite_density_is_rejected() {
        assert_eq!(density(f64::NAN), Err(InputError::NonFiniteDensity));
        assert_eq!(density(f64::INFINITY), Err(InputError::NonFiniteDensity));
    }

    #[test]
    fn invalid_reactivities_are_rejected() {
        assert_eq!(sv(0.0), Err(InputError::NonPositiveReactivity));
        assert_eq!(sv(-1.0), Err(InputError::NonPositiveReactivity));
        assert_eq!(sv(f64::NAN), Err(InputError::NonFiniteReactivity));
    }

    #[test]
    fn invalid_q_values_are_rejected() {
        assert_eq!(q(0.0), Err(InputError::NonPositiveEnergy));
        assert_eq!(q(-1.0), Err(InputError::NonPositiveEnergy));
        assert_eq!(q(f64::NAN), Err(InputError::NonFiniteEnergy));
    }

    #[test]
    fn wrong_species_for_channel_is_rejected() {
        let result = account_channel(
            ReactionChannel::Dt,
            distinct(Species::Tritium, 1.0e20, Species::Deuterium, 1.0e20),
            sv(1.0e-22).unwrap(),
            q(17.6).unwrap(),
        );
        assert_eq!(result, Err(InputError::FirstSpeciesMismatch));
    }

    #[test]
    fn identical_channel_rejects_distinct_density_shape() {
        let result = account_channel(
            ReactionChannel::DdNeutron,
            distinct(Species::Deuterium, 1.0e20, Species::Deuterium, 1.0e20),
            sv(1.0e-24).unwrap(),
            q(3.269).unwrap(),
        );
        assert_eq!(result, Err(InputError::ExpectedIdenticalReactants));
    }

    #[test]
    fn unlike_channel_rejects_identical_density_shape() {
        let result = account_channel(
            ReactionChannel::Dt,
            identical_d(Species::Deuterium, 1.0e20),
            sv(1.0e-24).unwrap(),
            q(17.6).unwrap(),
        );
        assert_eq!(result, Err(InputError::ExpectedDistinctReactants));
    }

    #[test]
    fn frozen_corpus_matches_oracle_dispositions() {
        const CORPUS: &str = include_str!("../fixtures/reaction-accounting-cases-v1.tsv");
        let mut checked = 0usize;

        for line in CORPUS.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line == "case_id|expected|mutation" {
                continue;
            }
            let fields: Vec<&str> = line.split('|').collect();
            assert_eq!(fields.len(), 3, "malformed row: {line}");
            let [case_id, expected, mutation] = [fields[0], fields[1], fields[2]];

            let outcome = match mutation {
                "dd_neutron" => account_channel(
                    ReactionChannel::DdNeutron,
                    identical_d(Species::Deuterium, 2.0e20),
                    sv(2.0e-24).unwrap(),
                    q(3.269).unwrap(),
                )
                .map(|_| ()),
                "dd_proton" => account_channel(
                    ReactionChannel::DdProton,
                    identical_d(Species::Deuterium, 2.0e20),
                    sv(3.0e-24).unwrap(),
                    q(4.033).unwrap(),
                )
                .map(|_| ()),
                "dd_total" => account_dd_total(density(2.0e20).unwrap(), dd_inputs()).map(|_| ()),
                "dt" => account_channel(
                    ReactionChannel::Dt,
                    distinct(Species::Deuterium, 1.0e20, Species::Tritium, 1.0e20),
                    sv(1.0e-22).unwrap(),
                    q(17.6).unwrap(),
                )
                .map(|_| ()),
                "dhe3" => account_channel(
                    ReactionChannel::DHe3,
                    distinct(Species::Deuterium, 1.0e20, Species::Helium3, 1.0e20),
                    sv(1.0e-23).unwrap(),
                    q(18.3).unwrap(),
                )
                .map(|_| ()),
                "pb11" => account_channel(
                    ReactionChannel::PB11,
                    distinct(Species::Proton, 1.0e20, Species::Boron11, 1.0e20),
                    sv(1.0e-25).unwrap(),
                    q(8.7).unwrap(),
                )
                .map(|_| ()),
                "he3he3" => account_channel(
                    ReactionChannel::He3He3,
                    identical_d(Species::Helium3, 1.0e20),
                    sv(1.0e-24).unwrap(),
                    q(12.9).unwrap(),
                )
                .map(|_| ()),
                "unlike_scale" => {
                    let base = account_channel(
                        ReactionChannel::Dt,
                        distinct(Species::Deuterium, 1.0e20, Species::Tritium, 1.0e20),
                        sv(1.0e-22).unwrap(),
                        q(17.6).unwrap(),
                    )
                    .unwrap();
                    let doubled = account_channel(
                        ReactionChannel::Dt,
                        distinct(Species::Deuterium, 2.0e20, Species::Tritium, 1.0e20),
                        sv(1.0e-22).unwrap(),
                        q(17.6).unwrap(),
                    )
                    .unwrap();
                    assert_eq!(doubled.event_rate.value(), 2.0 * base.event_rate.value());
                    Ok(())
                }
                "identical_scale" => {
                    let base = account_channel(
                        ReactionChannel::DdNeutron,
                        identical_d(Species::Deuterium, 1.0e20),
                        sv(1.0e-24).unwrap(),
                        q(3.269).unwrap(),
                    )
                    .unwrap();
                    let doubled = account_channel(
                        ReactionChannel::DdNeutron,
                        identical_d(Species::Deuterium, 2.0e20),
                        sv(1.0e-24).unwrap(),
                        q(3.269).unwrap(),
                    )
                    .unwrap();
                    assert_eq!(doubled.event_rate.value(), 4.0 * base.event_rate.value());
                    Ok(())
                }
                "branch_q" => {
                    let result = account_dd_total(density(1.0e20).unwrap(), dd_inputs()).unwrap();
                    let expected_power =
                        1.0e40 * (2.0e-24 * 3.269 + 3.0e-24 * 4.033) / 2.0 * MEV_TO_JOULE;
                    assert!(
                        (result.total_power_density.value() - expected_power).abs()
                            < expected_power * 1.0e-14
                    );
                    Ok(())
                }
                "branch_sum" => {
                    let result = account_dd_total(density(1.0e20).unwrap(), dd_inputs()).unwrap();
                    assert_eq!(
                        result.total_event_rate.value(),
                        result.neutron_branch.event_rate.value()
                            + result.proton_branch.event_rate.value()
                    );
                    Ok(())
                }
                "quarter_factor" => {
                    let identical = account_channel(
                        ReactionChannel::DdNeutron,
                        identical_d(Species::Deuterium, 1.0e20),
                        sv(1.0e-24).unwrap(),
                        q(3.269).unwrap(),
                    )
                    .unwrap();
                    let unlike = account_channel(
                        ReactionChannel::Dt,
                        distinct(Species::Deuterium, 1.0e20, Species::Tritium, 1.0e20),
                        sv(1.0e-24).unwrap(),
                        q(17.6).unwrap(),
                    )
                    .unwrap();
                    assert_ne!(identical.event_rate.value(), 1.0e40 * 1.0e-24 / 4.0);
                    assert_ne!(unlike.event_rate.value(), 1.0e40 * 1.0e-24 / 4.0);
                    Ok(())
                }
                "negative_density" => density(-1.0).map(|_| ()),
                "zero_density" => density(0.0).map(|_| ()),
                "nan_density" => density(f64::NAN).map(|_| ()),
                "infinite_density" => density(f64::INFINITY).map(|_| ()),
                "zero_reactivity" => sv(0.0).map(|_| ()),
                "negative_reactivity" => sv(-1.0).map(|_| ()),
                "nan_reactivity" => sv(f64::NAN).map(|_| ()),
                "zero_q" => q(0.0).map(|_| ()),
                "negative_q" => q(-1.0).map(|_| ()),
                "nan_q" => q(f64::NAN).map(|_| ()),
                "wrong_species" => account_channel(
                    ReactionChannel::Dt,
                    distinct(Species::Tritium, 1.0e20, Species::Deuterium, 1.0e20),
                    sv(1.0e-22).unwrap(),
                    q(17.6).unwrap(),
                )
                .map(|_| ()),
                "wrong_shape" => account_channel(
                    ReactionChannel::DdNeutron,
                    distinct(Species::Deuterium, 1.0e20, Species::Deuterium, 1.0e20),
                    sv(1.0e-24).unwrap(),
                    q(3.269).unwrap(),
                )
                .map(|_| ()),
                other => panic!("unknown oracle corpus mutation {other} for {case_id}"),
            };

            let actual = if outcome.is_ok() { "ACCEPT" } else { "REJECT" };
            assert_eq!(actual, expected, "case {case_id} / {mutation}");
            checked += 1;
        }
        assert_eq!(checked, 24);
    }
}
