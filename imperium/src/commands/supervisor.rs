use std::io::Read;

use clap::{Args, Subcommand};
use reqwest::blocking::Client;
use shared_types::{
    scheduling_environment::worker_environment::resources::{Id, MainResources},
    supervisor::{
        supervisor_scheduling_message::SupervisorSchedulingMessage,
        supervisor_status_message::SupervisorStatusMessage, SupervisorRequest,
        SupervisorRequestMessage,
    },
    Asset, SystemMessages,
};

#[derive(Subcommand, Debug)]
pub enum SupervisorCommands {
    /// Get the status of a SupervisorAgent
    Status {
        asset: Asset,
        supervisor: MainResources,
    },
    /// Get the commands for manually scheduling a work order activity.
    Scheduling {
        asset: Asset,
        supervisor: MainResources,
        #[clap(subcommand)]
        scheduling_commands: SchedulingCommands,
    },
    /// Test the Feasibility of the SupervisorAgent
    Test {
        asset: Asset,
        supervisor: MainResources,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingCommands {
    /// Schedule a specific work order activity to an operational agent
    Schedule(Assign),
}

#[derive(Args, Debug)]
pub struct Assign {
    work_order_number: u64,
    activity_number: u64,
    id_operational: String,
}

impl SupervisorCommands {
    pub fn execute(&self, client: &Client) -> SystemMessages {
        match self {
            SupervisorCommands::Status { asset, supervisor } => {
                let supervisor_status_message = SupervisorStatusMessage::General;

                let supervisor_request_message =
                    SupervisorRequestMessage::Status(supervisor_status_message);

                let supervisor_request = SupervisorRequest {
                    asset: asset.clone(),
                    main_work_center: supervisor.clone(),
                    supervisor_request_message,
                };

                SystemMessages::Supervisor(supervisor_request)
            }
            SupervisorCommands::Scheduling {
                asset,
                supervisor,
                scheduling_commands,
            } => match scheduling_commands {
                SchedulingCommands::Schedule(assign) => {
                    let id_operational = get_id_operational(client, assign.id_operational.clone());

                    let supervisor_scheduling_message = SupervisorSchedulingMessage::new(
                        (
                            assign.work_order_number.into(),
                            assign.activity_number.into(),
                        ),
                        id_operational,
                    );

                    let supervisor_request_message =
                        SupervisorRequestMessage::Scheduling(supervisor_scheduling_message);

                    let supervisor_request = SupervisorRequest {
                        asset: asset.clone(),
                        main_work_center: supervisor.clone(),
                        supervisor_request_message,
                    };

                    SystemMessages::Supervisor(supervisor_request)
                }
            },
            SupervisorCommands::Test { asset, supervisor } => {
                let supervisor_request_message = SupervisorRequestMessage::Test;

                let supervisor_request = SupervisorRequest {
                    asset: asset.clone(),
                    main_work_center: supervisor.clone(),
                    supervisor_request_message,
                };

                SystemMessages::Supervisor(supervisor_request)
            }
        }
    }
}

fn get_id_operational(client: &Client, id_operational: String) -> Id {
    let url: String = "http://".to_string() + &dotenvy::var("ORDINATOR_API_ADDRESS").unwrap() + &dotenvy::var("ORDINATOR_MAIN_ENDPOINT)").unwrap();

    let mut id_operational_json = String::new();
    client
        .get(url)
        .body(id_operational)
        .send()
        .unwrap()
        .read_to_string(&mut id_operational_json)
        .unwrap();

    let id_operational: Id = serde_json::from_str(&id_operational_json).unwrap();
    id_operational
}
