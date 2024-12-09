pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

use crate::scheduling_environment::worker_environment::resources::Resources;
use crate::strategic::{Periods, StrategicResources};
use crate::tactical::{Days, TacticalResources};
use crate::SystemAgents;

use super::time_environment::day::Day;
use super::time_environment::period::Period;
use super::work_order::operation::Work;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct WorkerEnvironment {
    pub system_agents: SystemAgents,
    work_centers: HashSet<Resources>,
}

impl WorkerEnvironment {
    pub fn new() -> Self {
        let mut work_centers = HashSet::new();
        for resource in Resources::iter() {
            work_centers.insert(resource);
        }
        WorkerEnvironment {
            system_agents: SystemAgents::default(),
            work_centers,
        }
    }

    pub fn get_work_centers(&self) -> &HashSet<Resources> {
        &self.work_centers
    }

    pub fn initialize_from_resource_configuration_file(
        &mut self,
        system_agents_bytes: Vec<u8>,
    ) -> Result<()> {
        let contents = std::str::from_utf8(&system_agents_bytes)
            .context("configuration file bitstream not read correct")?;

        let system_agents: SystemAgents = toml::from_str(contents)
            .with_context(|| format!("configuration file string could not be parsed into {}. Likely a toml parsing error", std::any::type_name::<SystemAgents>().bright_red()))?;

        self.system_agents = system_agents;
        Ok(())
    }

    pub fn generate_strategic_resources(&self, periods: &[Period]) -> StrategicResources {
        let _hours_per_day = 6.0;
        let days_in_period = 13.0;

        let gradual_reduction = |i: usize| -> f64 {
            if i == 0 {
                1.0
            } else if i == 1 {
                0.9
            } else if i == 2 {
                0.8
            } else {
                0.6
            }
        };

        let mut strategic_resources_inner = HashMap::<Resources, Periods>::new();

        for operational_agent in &self.system_agents.operational {
            for (i, period) in periods.iter().enumerate() {
                let resource_periods = strategic_resources_inner
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Periods(HashMap::new()));

                *resource_periods
                    .0
                    .entry(period.clone())
                    .or_insert_with(|| Work::from(0.0)) += Work::from(
                    operational_agent.hours_per_day * days_in_period * gradual_reduction(i),
                )
            }
        }

        StrategicResources::new(strategic_resources_inner)
    }

    pub fn generate_tactical_resources(&self, days: &[Day]) -> TacticalResources {
        let _hours_per_day = 6.0;

        let gradual_reduction = |i: usize| -> f64 {
            match i {
                0..=13 => 1.0,
                14..=27 => 1.0,
                _ => 1.0,
            }
        };

        let mut tactical_resources_inner = HashMap::<Resources, Days>::new();
        for operational_agent in &self.system_agents.operational {
            for (i, day) in days.iter().enumerate() {
                let resource_periods = tactical_resources_inner
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Days::new(HashMap::new()));

                *resource_periods
                    .days
                    .entry(day.clone())
                    .or_insert_with(|| Work::from(0.0)) +=
                    Work::from(operational_agent.hours_per_day * gradual_reduction(i));
            }
        }
        TacticalResources::new(tactical_resources_inner)
    }
}
