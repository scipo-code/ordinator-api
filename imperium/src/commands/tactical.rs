use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use shared_messages::{
    resources::Resources,
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
) -> HashMap<Resources, HashMap<String, f64>> {
    let periods: Vec<String> = orchestrator::tactical_days(client);
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
            Resources::Medic => config.Medic * hours_per_day, //50.0,
            Resources::MtnCran => config.MtnCran * hours_per_day,
            Resources::MtnElec => config.MtnElec * hours_per_day,
            Resources::MtnInst => config.MtnInst * hours_per_day,
            Resources::MtnLagg => config.MtnLagg * hours_per_day, //300.0,
            Resources::MtnMech => config.MtnMech * hours_per_day,
            Resources::MtnPain => config.MtnPain * hours_per_day, //300.0,
            Resources::MtnPipf => config.MtnPipf * hours_per_day, //300.0,
            Resources::MtnRigg => config.MtnRigg * hours_per_day, //300.0,
            Resources::MtnRope => config.MtnRope * hours_per_day, //300.0,
            Resources::MtnRous => config.MtnRous * hours_per_day, //300.0,
            Resources::MtnSat => config.MtnSat * hours_per_day,   //300.0,
            Resources::MtnScaf => config.MtnScaf * hours_per_day, //300.0,
            Resources::MtnTele => config.MtnTele * hours_per_day,
            Resources::MtnTurb => config.MtnTurb * hours_per_day,
            Resources::InpSite => config.InpSite * hours_per_day,
            Resources::Prodlabo => config.Prodlabo * hours_per_day, //300.0,
            Resources::Prodtech => config.Prodtech * hours_per_day,
            Resources::VenAcco => config.VenAcco * hours_per_day, //300.0,
            Resources::VenComm => config.VenComm * hours_per_day, //300.0,
            Resources::VenCran => config.VenCran * hours_per_day, //300.0,
            Resources::VenElec => config.VenElec * hours_per_day, //300.0,
            Resources::VenHvac => config.VenHvac * hours_per_day, //300.0,
            Resources::VenInsp => config.VenInsp * hours_per_day, //300.0,
            Resources::VenInst => config.VenInst * hours_per_day, //300.0,
            Resources::VenMech => config.VenMech * hours_per_day, //300.0,
            Resources::VenMete => config.VenMete * hours_per_day, //300.0,
            Resources::VenRope => config.VenRope * hours_per_day, //300.0,
            Resources::VenScaf => config.VenScaf * hours_per_day, //300.0,
            Resources::VenSubs => config.VenSubs * hours_per_day, //300.0,
            Resources::QaqcElec => config.QaqcElec * hours_per_day, //300.0,
            Resources::QaqcMech => config.QaqcMech * hours_per_day, //300.0,
            Resources::QaqcPain => config.QaqcPain * hours_per_day, //300.0,
            Resources::WellSupv => config.WellSupv * hours_per_day, //300.0,
        }
    };

    let mut resources_hash_map = HashMap::new();
    for resource in shared_messages::resources::Resources::iter() {
        let mut periods_hash_map = HashMap::new();
        for (i, period) in periods.clone().iter().enumerate() {
            periods_hash_map.insert(
                period.to_string(),
                resource_specific(&resource) * gradual_reduction(i),
            );
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    resources_hash_map
}

// What will the goal be for now.
