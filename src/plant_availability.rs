//! First-order reliability/availability screening for fusion-plant component groups.
//!
//! This is deliberately separate from Q_econ's fluence-limited surface
//! utilization. It is a deterministic steady-state availability approximation,
//! not a stochastic RAMI simulator, reliability prediction, or commercial
//! availability guarantee.
//!
//! Each component's long-run availability is estimated as MTBF/(MTBF+MTTR).
//! Within a group, component up/down states are treated as independent and the
//! group is available when at least N of M components are online. Groups are
//! then treated as a series system: every declared group must be available.
//!
//! Planned-maintenance and common-cause availability factors are explicit
//! system-level factors. Multiplication assumes these factors are conditionally
//! independent of the component/group availability already calculated and of
//! one another. Callers must not supply overlapping downtime in multiple
//! factors. If these assumptions are not defensible, this screening result
//! should not be used; the next model increment must represent event timelines,
//! dependencies and common-cause events explicitly.
//!
//! Overall plant availability is withheld unless the caller declares that all
//! critical systems have been included. That declaration is a caller assertion,
//! not an independently verified fact.

use serde::{Deserialize, Serialize};

/// MTBF and corrective-repair profile for one maintainable component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentRepairProfile {
    /// Stable component identifier within the declared plant boundary.
    pub component_id: String,
    /// Mean operating time between failures, in hours.
    pub mean_time_between_failures_hours: f64,
    /// Mean corrective repair duration, in hours.
    ///
    /// This should include diagnosis, access, removal, repair/refurbishment,
    /// reinstallation and return-to-service qualification for this component.
    /// Scheduled inspection/maintenance is represented separately.
    pub mean_time_to_repair_hours: f64,
}

/// A redundant or non-redundant group of components.
///
/// The group is available when at least `minimum_online_components` of the
/// declared component set are available. This captures a series component
/// (`N=1, M=1`), a fully redundant pair (`N=1, M=2`), or other N-of-M groups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilityGroup {
    pub group_id: String,
    pub minimum_online_components: usize,
    pub components: Vec<ComponentRepairProfile>,
}

/// Declared assumptions and profiles for a first-order plant availability run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlantAvailabilityInput {
    /// Every group is required for the declared plant function.
    pub required_groups: Vec<AvailabilityGroup>,
    /// Fraction of calendar time available after planned outage allowance.
    ///
    /// Apply planned outages once. If planned maintenance is performed during
    /// corrective outages, the input must represent the incremental planned
    /// downtime conditional on the component-availability model rather than
    /// simply subtracting all planned hours again.
    pub planned_maintenance_availability_factor: f64,
    /// Conditional system-level availability factor for explicitly modelled
    /// common-cause outages, after excluding overlap with component failures
    /// and planned maintenance. This is not an estimated common-cause rate.
    pub common_cause_availability_factor: f64,
    /// True only when all critical systems required for the plant function are
    /// represented in `required_groups`.
    pub all_critical_systems_accounted_for: bool,
}

/// Component availability derived from its MTBF and MTTR inputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentAvailabilityResult {
    pub component_id: String,
    pub steady_state_availability: f64,
}

/// Reliability result for one N-of-M group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilityGroupResult {
    pub group_id: String,
    pub minimum_online_components: usize,
    pub component_results: Vec<ComponentAvailabilityResult>,
    /// Probability the group is available under the independent steady-state
    /// component model.
    pub group_availability: f64,
}

/// Deterministic screening result for plant-level availability.
///
/// `independent_group_system_availability` accounts for only the supplied
/// component groups. `total_calendar_availability` additionally applies the
/// explicit planned-maintenance and common-cause factors and is present only
/// when the caller declares a complete critical-system boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlantAvailabilityResult {
    pub group_results: Vec<AvailabilityGroupResult>,
    pub independent_group_system_availability: f64,
    pub planned_maintenance_availability_factor: f64,
    pub common_cause_availability_factor: f64,
    pub total_calendar_availability: Option<f64>,
    pub critical_system_boundary_complete: bool,
    pub model_revision: String,
    pub assumptions: Vec<String>,
}

/// Revision label for this deterministic screening model.
pub const AVAILABILITY_MODEL_REVISION: &str = "series-n-of-m-availability-v0.1";

/// Calculate a steady-state series-system availability from explicit profiles.
///
/// Component steady-state availability is MTBF/(MTBF+MTTR). Each group uses
/// the Poisson-binomial distribution to calculate P(number online >= N), so
/// individual components may have different MTBF and MTTR values.
pub fn calculate_plant_availability(
    input: &PlantAvailabilityInput,
) -> Result<PlantAvailabilityResult, String> {
    validate_input(input)?;

    let mut group_results = Vec::with_capacity(input.required_groups.len());
    let mut independent_group_system_availability = 1.0;

    for group in &input.required_groups {
        let component_results: Vec<ComponentAvailabilityResult> = group
            .components
            .iter()
            .map(|component| {
                let availability = component.mean_time_between_failures_hours
                    / (component.mean_time_between_failures_hours
                        + component.mean_time_to_repair_hours);
                ComponentAvailabilityResult {
                    component_id: component.component_id.clone(),
                    steady_state_availability: availability,
                }
            })
            .collect();

        let component_availabilities: Vec<f64> = component_results
            .iter()
            .map(|component| component.steady_state_availability)
            .collect();
        let group_availability = probability_at_least_n_available(
            &component_availabilities,
            group.minimum_online_components,
        );
        if !group_availability.is_finite() || !(0.0..=1.0).contains(&group_availability) {
            return Err(format!(
                "calculated availability for group {} is outside [0, 1]",
                group.group_id
            ));
        }

        independent_group_system_availability *= group_availability;
        group_results.push(AvailabilityGroupResult {
            group_id: group.group_id.clone(),
            minimum_online_components: group.minimum_online_components,
            component_results,
            group_availability,
        });
    }

    if !independent_group_system_availability.is_finite()
        || !(0.0..=1.0).contains(&independent_group_system_availability)
    {
        return Err("calculated system availability is outside [0, 1]".to_string());
    }

    let total_calendar_availability = if input.all_critical_systems_accounted_for {
        Some(
            independent_group_system_availability
                * input.planned_maintenance_availability_factor
                * input.common_cause_availability_factor,
        )
    } else {
        None
    };

    Ok(PlantAvailabilityResult {
        group_results,
        independent_group_system_availability,
        planned_maintenance_availability_factor: input
            .planned_maintenance_availability_factor,
        common_cause_availability_factor: input.common_cause_availability_factor,
        total_calendar_availability,
        critical_system_boundary_complete: input.all_critical_systems_accounted_for,
        model_revision: AVAILABILITY_MODEL_REVISION.to_string(),
        assumptions: vec![
            "Steady-state component availability is MTBF/(MTBF+MTTR).".to_string(),
            "Component up/down states are independent within each N-of-M group.".to_string(),
            "Every declared group is required; groups form a series system.".to_string(),
            "Planned-maintenance and common-cause factors are explicit and conditionally independent of modelled component downtime and of each other.".to_string(),
            "The caller's critical-system-boundary declaration is not independently verified.".to_string(),
            "This deterministic screening model does not represent repair queues, time-dependent maintenance schedules, common-cause event trees, or uncertainty distributions.".to_string(),
        ],
    })
}

fn probability_at_least_n_available(
    component_availabilities: &[f64],
    minimum_online_components: usize,
) -> f64 {
    // Poisson-binomial dynamic program. At each step, probabilities[k]
    // represents exactly k components online after the processed inputs.
    let mut exactly_online = vec![0.0; component_availabilities.len() + 1];
    exactly_online[0] = 1.0;
    let mut processed = 0_usize;

    for availability in component_availabilities {
        for count in (0..=processed + 1).rev() {
            let previous_same = if count <= processed {
                exactly_online[count]
            } else {
                0.0
            };
            let previous_one_fewer = if count > 0 {
                exactly_online[count - 1]
            } else {
                0.0
            };
            exactly_online[count] =
                previous_same * (1.0 - availability) + previous_one_fewer * availability;
        }
        processed += 1;
    }

    exactly_online[minimum_online_components..]
        .iter()
        .sum::<f64>()
}

fn validate_input(input: &PlantAvailabilityInput) -> Result<(), String> {
    if input.required_groups.is_empty() {
        return Err("required_groups must not be empty".to_string());
    }
    validate_fraction(
        "planned_maintenance_availability_factor",
        input.planned_maintenance_availability_factor,
    )?;
    validate_fraction(
        "common_cause_availability_factor",
        input.common_cause_availability_factor,
    )?;

    let mut group_ids = std::collections::HashSet::new();
    let mut component_ids = std::collections::HashSet::new();
    for group in &input.required_groups {
        if group.group_id.trim().is_empty() {
            return Err("group_id must not be empty".to_string());
        }
        if !group_ids.insert(group.group_id.as_str()) {
            return Err(format!("duplicate group_id: {}", group.group_id));
        }
        if group.components.is_empty() {
            return Err(format!("group {} has no components", group.group_id));
        }
        if group.minimum_online_components == 0
            || group.minimum_online_components > group.components.len()
        {
            return Err(format!(
                "group {} minimum_online_components must be in 1..=component_count",
                group.group_id
            ));
        }

        for component in &group.components {
            if component.component_id.trim().is_empty() {
                return Err("component_id must not be empty".to_string());
            }
            if !component_ids.insert(component.component_id.as_str()) {
                return Err(format!(
                    "duplicate component_id across plant boundary: {}",
                    component.component_id
                ));
            }
            if !component.mean_time_between_failures_hours.is_finite()
                || component.mean_time_between_failures_hours <= 0.0
            {
                return Err(format!(
                    "{} mean_time_between_failures_hours must be finite and positive",
                    component.component_id
                ));
            }
            if !component.mean_time_to_repair_hours.is_finite()
                || component.mean_time_to_repair_hours < 0.0
            {
                return Err(format!(
                    "{} mean_time_to_repair_hours must be finite and non-negative",
                    component.component_id
                ));
            }
            if !(component.mean_time_between_failures_hours
                + component.mean_time_to_repair_hours)
                .is_finite()
            {
                return Err(format!(
                    "{} MTBF + MTTR must be finite",
                    component.component_id
                ));
            }
        }
    }
    Ok(())
}

fn validate_fraction(name: &str, value: f64) -> Result<(), String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(format!("{name} must be finite and in [0, 1]"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(id: &str, mtbf: f64, mttr: f64) -> ComponentRepairProfile {
        ComponentRepairProfile {
            component_id: id.to_string(),
            mean_time_between_failures_hours: mtbf,
            mean_time_to_repair_hours: mttr,
        }
    }

    fn input() -> PlantAvailabilityInput {
        PlantAvailabilityInput {
            required_groups: vec![AvailabilityGroup {
                group_id: "primary-cooling".to_string(),
                minimum_online_components: 1,
                components: vec![component("pump-a", 90.0, 10.0)],
            }],
            planned_maintenance_availability_factor: 1.0,
            common_cause_availability_factor: 1.0,
            all_critical_systems_accounted_for: true,
        }
    }

    #[test]
    fn single_component_uses_mtbf_over_mtbf_plus_mttr() {
        let result = calculate_plant_availability(&input()).unwrap();
        assert!((result.total_calendar_availability.unwrap() - 0.9).abs() < 1e-12);
    }

    #[test]
    fn heterogeneous_n_of_m_group_uses_poisson_binomial_probability() {
        let mut scenario = input();
        scenario.required_groups[0].minimum_online_components = 1;
        scenario.required_groups[0].components = vec![
            component("pump-a", 90.0, 10.0), // A = 0.9
            component("pump-b", 80.0, 20.0), // A = 0.8
        ];
        let result = calculate_plant_availability(&scenario).unwrap();
        // P(at least one available) = 1 - (0.1 * 0.2) = 0.98.
        assert!((result.group_results[0].group_availability - 0.98).abs() < 1e-12);
    }

    #[test]
    fn n_of_m_group_can_require_multiple_online_components() {
        let mut scenario = input();
        scenario.required_groups[0].minimum_online_components = 2;
        scenario.required_groups[0].components = vec![
            component("pump-a", 90.0, 10.0),
            component("pump-b", 90.0, 10.0),
            component("pump-c", 90.0, 10.0),
        ];
        let result = calculate_plant_availability(&scenario).unwrap();
        // For three A=0.9 components, P(at least two) = 3(0.9²)(0.1)+0.9³.
        assert!((result.group_results[0].group_availability - 0.972).abs() < 1e-12);
    }

    #[test]
    fn system_series_groups_and_explicit_downtime_factors_are_applied_once() {
        let mut scenario = input();
        scenario.required_groups.push(AvailabilityGroup {
            group_id: "vacuum".to_string(),
            minimum_online_components: 1,
            components: vec![component("vacuum-pump", 80.0, 20.0)], // A = 0.8
        });
        scenario.planned_maintenance_availability_factor = 0.9;
        scenario.common_cause_availability_factor = 0.95;
        let result = calculate_plant_availability(&scenario).unwrap();
        // 0.9 * 0.8 * 0.9 planned * 0.95 common cause.
        assert!((result.total_calendar_availability.unwrap() - 0.6156).abs() < 1e-12);
    }

    #[test]
    fn incomplete_critical_system_boundary_withholds_total_availability() {
        let mut scenario = input();
        scenario.all_critical_systems_accounted_for = false;
        let result = calculate_plant_availability(&scenario).unwrap();
        assert!(result.total_calendar_availability.is_none());
        assert_eq!(result.independent_group_system_availability, 0.9);
    }

    #[test]
    fn zero_repair_time_yields_unit_component_availability() {
        let mut scenario = input();
        scenario.required_groups[0].components[0] = component("ideal", 1.0, 0.0);
        let result = calculate_plant_availability(&scenario).unwrap();
        assert_eq!(result.total_calendar_availability, Some(1.0));
    }

    #[test]
    fn rejects_invalid_profiles_and_duplicate_ids() {
        let mut scenario = input();
        scenario.required_groups[0].components[0].mean_time_between_failures_hours = 0.0;
        assert!(calculate_plant_availability(&scenario).is_err());

        let mut scenario = input();
        scenario.required_groups[0].components.push(component("pump-a", 90.0, 10.0));
        assert!(calculate_plant_availability(&scenario).is_err());

        let mut scenario = input();
        scenario.required_groups[0].minimum_online_components = 2;
        assert!(calculate_plant_availability(&scenario).is_err());
    }
}
