use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use shared_messages::{
    resources::Resources,
    tactical::{
        tactical_resources_message::TacticalResourceMessage,
        tactical_status_message::TacticalStatusMessage, TacticalRequest,
    },
    SystemMessages,
};
use strum::IntoEnumIterator;

use super::orchestrator;

#[derive(Subcommand, Debug)]
pub enum TacticalCommands {
    /// Get the status of the tactical agent
    Status,
    /// Get the objectives of the tactical agent
    Resources {
        #[clap(subcommand)]
        resource_commands: ResourceCommands,
    },
    /// Access the scheduling of the tactical agent
    Scheduling,
    /// Access the days of the tactical agent
    Days,
}

impl TacticalCommands {
    pub fn execute(&self, client: &Client) -> shared_messages::SystemMessages {
        match self {
            TacticalCommands::Status => {
                dbg!("TacticalAgent Status Message");

                SystemMessages::Tactical(TacticalRequest::Status(TacticalStatusMessage::General))
            }

            TacticalCommands::Resources { resource_commands } => {
                dbg!("TacticalAgent Resources Message");

                // Here we want to match the subcommand and fill in the capacity for the tactical
                // agent. Okay I think that we should calm down a little here and consider what our

                match resource_commands {
                    ResourceCommands::GetCapacities {
                        days_end,
                        select_resources,
                    } => {
                        let tactical_resources_message = TacticalResourceMessage::GetCapacities {
                            days_end: days_end.to_string(),
                            select_resources: select_resources.clone(),
                        };

                        let tactical_request =
                            TacticalRequest::Resources(tactical_resources_message);

                        SystemMessages::Tactical(tactical_request)
                    }
                    ResourceCommands::GetLoadings {
                        days_end,
                        select_resources,
                    } => {
                        let tactical_resources_message = TacticalResourceMessage::GetLoadings {
                            days_end: days_end.to_string(),
                            select_resources: select_resources.clone(),
                        };

                        let tactical_request =
                            TacticalRequest::Resources(tactical_resources_message);

                        SystemMessages::Tactical(tactical_request)
                    }
                    ResourceCommands::SetCapacityPolicyDefault => {
                        let resources = generate_manual_resources(client);

                        let tactical_resources_message =
                            TacticalResourceMessage::new_set_resources(resources);

                        let tactical_request =
                            TacticalRequest::Resources(tactical_resources_message);

                        SystemMessages::Tactical(tactical_request)
                    }
                }
            }
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
    GetLoadings {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    /// Set a default capacity (USED FOR TESTING)
    SetCapacityPolicyDefault,
}

/// I will need to generate the manual resources for the tactical agent.
fn generate_manual_resources(client: &Client) -> HashMap<Resources, HashMap<String, f64>> {
    let periods: Vec<String> = orchestrator::tactical_days(client);

    let gradual_reduction = |i: usize| -> f64 {
        match i {
            0..=13 => 1.0,
            14..=27 => 0.8,
            _ => 0.6,
        }
    };

    let resource_specific = |resource: &Resources| -> f64 {
        match resource {
            Resources::Medic => 0.0, //50.0,
            Resources::MtnCran => 5.0,
            Resources::MtnElec => 12.0,
            Resources::MtnInst => 12.0,
            Resources::MtnLagg => 0.0, //300.0,
            Resources::MtnMech => 25.0,
            Resources::MtnPain => 0.0,  //300.0,
            Resources::MtnPipf => 0.0,  //300.0,
            Resources::MtnRigg => 14.0, //300.0,
            Resources::MtnRope => 0.0,  //300.0,
            Resources::MtnRous => 0.0,  //300.0,
            Resources::MtnSat => 0.0,   //300.0,
            Resources::MtnScaf => 14.0, //300.0,
            Resources::MtnTele => 12.0,
            Resources::MtnTurb => 6.0,
            Resources::InpSite => 21.0,
            Resources::Prodlabo => 0.0, //300.0,
            Resources::Prodtech => 13.0,
            Resources::VenAcco => 0.0,  //300.0,
            Resources::VenComm => 0.0,  //300.0,
            Resources::VenCran => 0.0,  //300.0,
            Resources::VenElec => 0.0,  //300.0,
            Resources::VenHvac => 0.0,  //300.0,
            Resources::VenInsp => 0.0,  //300.0,
            Resources::VenInst => 0.0,  //300.0,
            Resources::VenMech => 0.0,  //300.0,
            Resources::VenMete => 0.0,  //300.0,
            Resources::VenRope => 0.0,  //300.0,
            Resources::VenScaf => 0.0,  //300.0,
            Resources::VenSubs => 0.0,  //300.0,
            Resources::QaqcElec => 0.0, //300.0,
            Resources::QaqcMech => 0.0, //300.0,
            Resources::QaqcPain => 0.0, //300.0,
            Resources::WellSupv => 0.0, //300.0,
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
