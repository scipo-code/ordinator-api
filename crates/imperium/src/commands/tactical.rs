use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use shared_types::{
    agents::tactical::{
        requests::{
            tactical_resources_message::TacticalResourceRequest,
            tactical_status_message::TacticalStatusMessage,
        },
        Days, TacticalRequest, TacticalRequestMessage, TacticalResources,
    },
    scheduling_environment::{
        time_environment::day::Day, work_order::operation::Work,
        worker_environment::resources::Resources,
    },
    Asset, SystemAgents, SystemMessages,
};

use super::orchestrator;

#[derive(Subcommand, Debug)]
pub enum TacticalCommands {
    /// Get the status of the tactical agent
    Status { asset: Asset },
    /// Get the objectives of the tactical agent
    Resources {
        asset: Asset,
        #[clap(subcommand)]
        resource_commands: ResourceCommands,
    },
    /// Access the scheduling of the tactical agent
    Scheduling,
    /// Access the days of the tactical agent
    Days,
}

impl TacticalCommands {
    pub fn execute(&self, client: &Client) -> shared_types::SystemMessages {
        match self {
            TacticalCommands::Status { asset } => {
                dbg!("TacticalAgent Status Message");

                let tactical_request = TacticalRequest {
                    asset: asset.clone(),
                    tactical_request_message: TacticalRequestMessage::Status(
                        TacticalStatusMessage::General,
                    ),
                };

                SystemMessages::Tactical(tactical_request)
            }

            TacticalCommands::Resources {
                asset,
                resource_commands,
            } => match resource_commands {
                ResourceCommands::Capacity {
                    days_end,
                    select_resources,
                } => {
                    let tactical_resources_message = TacticalResourceRequest::GetCapacities {
                        days_end: days_end.to_string(),
                        select_resources: select_resources.clone(),
                    };

                    let tactical_request_request =
                        TacticalRequestMessage::Resources(tactical_resources_message);

                    let tactical_request = TacticalRequest {
                        asset: asset.clone(),
                        tactical_request_message: tactical_request_request,
                    };

                    SystemMessages::Tactical(tactical_request)
                }
                ResourceCommands::Loading {
                    days_end,
                    select_resources,
                } => {
                    let tactical_resources_message = TacticalResourceRequest::GetLoadings {
                        days_end: days_end.to_string(),
                        select_resources: select_resources.clone(),
                    };

                    let tactical_request_message =
                        TacticalRequestMessage::Resources(tactical_resources_message);

                    let tactical_request = TacticalRequest {
                        asset: asset.clone(),
                        tactical_request_message,
                    };

                    SystemMessages::Tactical(tactical_request)
                }
                ResourceCommands::PercentageLoading {
                    days_end,
                    select_resources,
                } => {
                    let tactical_resources_message =
                        TacticalResourceRequest::GetPercentageLoadings {
                            days_end: days_end.to_string(),
                            resources: select_resources.clone(),
                        };

                    let tactical_request_message =
                        TacticalRequestMessage::Resources(tactical_resources_message);

                    let tactical_request = TacticalRequest {
                        asset: asset.clone(),
                        tactical_request_message,
                    };
                    SystemMessages::Tactical(tactical_request)
                }
                ResourceCommands::LoadCapacityFile { toml_path } => {
                    let resources = generate_manual_resources(client, toml_path.clone());

                    let tactical_resources = resources;
                    let tactical_resources_message =
                        TacticalResourceRequest::new_set_resources(tactical_resources);

                    let tactical_request_message =
                        TacticalRequestMessage::Resources(tactical_resources_message);
                    let tactical_request = TacticalRequest {
                        asset: asset.clone(),
                        tactical_request_message,
                    };
                    SystemMessages::Tactical(tactical_request)
                }
            },
            TacticalCommands::Scheduling => {
                todo!()
            }
            TacticalCommands::Days => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ResourceCommands {
    Loading {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    Capacity {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    PercentageLoading {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    /// Set a capacity based on a file
    LoadCapacityFile { toml_path: String },
}

/// I will need to generate the manual resources for the tactical agent.
fn generate_manual_resources(client: &Client, toml_path: String) -> TacticalResources {
    let days: Vec<Day> = orchestrator::tactical_days(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();

    let config: SystemAgents = toml::from_str(&contents).unwrap();

    let _hours_per_day = 6.0;

    let gradual_reduction = |i: usize| -> f64 {
        match i {
            0..=13 => 1.0,
            14..=27 => 1.0,
            _ => 1.0,
        }
    };

    let mut resources_hash_map = HashMap::<Resources, Days>::new();
    for operational_agent in config.operational {
        for (i, day) in days.clone().iter().enumerate() {
            let resource_periods = resources_hash_map
                .entry(operational_agent.resources.first().cloned().unwrap())
                .or_insert(Days::new(HashMap::new()));

            *resource_periods.days.entry(day.clone()).or_insert_with(|| {
                Work::from(operational_agent.hours_per_day * gradual_reduction(i))
            }) += Work::from(operational_agent.hours_per_day * gradual_reduction(i))
        }
    }
    TacticalResources::new(resources_hash_map)
}
