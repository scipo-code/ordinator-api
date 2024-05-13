use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use shared_messages::{
    models::{time_environment::day::Day, worker_environment::resources::Resources},
    tactical::{
        tactical_resources_message::TacticalResourceMessage,
        tactical_status_message::TacticalStatusMessage, TacticalRequest, TacticalRequestMessage,
    },
    Asset, SystemMessages, TomlResources,
};
use strum::IntoEnumIterator;

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
    /// Test the feasibility of the tactical schedule
    Test { asset: Asset },
}

impl TacticalCommands {
    pub fn execute(&self, client: &Client) -> shared_messages::SystemMessages {
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
                    let tactical_resources_message = TacticalResourceMessage::GetCapacities {
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
                    let tactical_resources_message = TacticalResourceMessage::GetLoadings {
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
                        TacticalResourceMessage::GetPercentageLoadings {
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

                    let tactical_resources_message =
                        TacticalResourceMessage::new_set_resources(resources);

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
            TacticalCommands::Test { asset } => {
                let tactical_request_message = TacticalRequestMessage::Test;

                let tactical_request = TacticalRequest {
                    asset: asset.clone(),
                    tactical_request_message,
                };
                SystemMessages::Tactical(tactical_request)
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
fn generate_manual_resources(
    client: &Client,
    toml_path: String,
) -> HashMap<Resources, HashMap<Day, f64>> {
    let days: Vec<Day> = orchestrator::tactical_days(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();

    let config: TomlResources = toml::de::from_str(&contents).unwrap();

    let hours_per_day = 6.0;

    let gradual_reduction = |i: usize| -> f64 {
        match i {
            0..=13 => 1.0,
            14..=27 => 1.0,
            _ => 1.0,
        }
    };

    let resource_specific = |resource: &Resources| -> f64 {
        match resource {
            Resources::Medic => config.medic * hours_per_day, //50.0,
            Resources::MtnCran => config.mtncran * hours_per_day,
            Resources::MtnElec => config.mtnelec * hours_per_day,
            Resources::MtnInst => config.mtninst * hours_per_day,
            Resources::MtnLagg => config.mtnlagg * hours_per_day, //300.0,
            Resources::MtnMech => config.mtnmech * hours_per_day,
            Resources::MtnPain => config.mtnpain * hours_per_day, //300.0,
            Resources::MtnPipf => config.mtnpipf * hours_per_day, //300.0,
            Resources::MtnRigg => config.mtnrigg * hours_per_day, //300.0,
            Resources::MtnRope => config.mtnrope * hours_per_day, //300.0,
            Resources::MtnRous => config.mtnrous * hours_per_day, //300.0,
            Resources::MtnSat => config.mtnsat * hours_per_day,   //300.0,
            Resources::MtnScaf => config.mtnscaf * hours_per_day, //300.0,
            Resources::MtnTele => config.mtntele * hours_per_day,
            Resources::MtnTurb => config.mtnturb * hours_per_day,
            Resources::InpSite => config.inpsite * hours_per_day,
            Resources::Prodlabo => config.prodlabo * hours_per_day, //300.0,
            Resources::Prodtech => config.prodtech * hours_per_day,
            Resources::VenAcco => config.venacco * hours_per_day, //300.0,
            Resources::VenComm => config.vencomm * hours_per_day, //300.0,
            Resources::VenCran => config.vencran * hours_per_day, //300.0,
            Resources::VenElec => config.venelec * hours_per_day, //300.0,
            Resources::VenHvac => config.venhvac * hours_per_day, //300.0,
            Resources::VenInsp => config.veninsp * hours_per_day, //300.0,
            Resources::VenInst => config.veninst * hours_per_day, //300.0,
            Resources::VenMech => config.venmech * hours_per_day, //300.0,
            Resources::VenMete => config.venmete * hours_per_day, //300.0,
            Resources::VenRope => config.venrope * hours_per_day, //300.0,
            Resources::VenScaf => config.venscaf * hours_per_day, //300.0,
            Resources::VenSubs => config.vensubs * hours_per_day, //300.0,
            Resources::QaqcElec => config.qaqcelec * hours_per_day, //300.0,
            Resources::QaqcMech => config.qaqcmech * hours_per_day, //300.0,
            Resources::QaqcPain => config.qaqcpain * hours_per_day, //300.0,
            Resources::WellSupv => config.wellsupv * hours_per_day, //300.0,
        }
    };

    let mut resources: HashMap<Resources, HashMap<Day, f64>> = HashMap::new();
    for resource in shared_messages::models::worker_environment::resources::Resources::iter() {
        let mut capacity = HashMap::new();
        for (i, day) in days.clone().iter().enumerate() {
            capacity.insert(
                day.clone(),
                resource_specific(&resource) * gradual_reduction(i),
            );
        }
        resources.insert(resource, capacity);
    }
    resources
}
