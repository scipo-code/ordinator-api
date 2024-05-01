use clap::Subcommand;
use shared_messages::{
    models::worker_environment::resources::MainResources,
    supervisor::{
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
    /// Test the Feasibility of the SupervisorAgent
    Test {
        asset: Asset,
        supervisor: MainResources,
    },
}

impl SupervisorCommands {
    pub fn execute(&self) -> SystemMessages {
        match self {
            SupervisorCommands::Status { asset, supervisor } => {
                let supervisor_status_message = SupervisorStatusMessage {};

                let supervisor_request_message =
                    SupervisorRequestMessage::Status(supervisor_status_message);

                let supervisor_request = SupervisorRequest {
                    asset: asset.clone(),
                    main_work_center: supervisor.clone(),
                    supervisor_request_message,
                };

                SystemMessages::Supervisor(supervisor_request)
            }
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
