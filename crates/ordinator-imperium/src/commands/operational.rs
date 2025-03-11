use clap::Subcommand;
use shared_types::{
    agents::operational::{
        requests::operational_request_scheduling::OperationalSchedulingRequest, OperationalRequest,
        OperationalRequestMessage,
    },
    Asset, SystemMessages,
};

#[derive(Subcommand, Debug)]
pub enum OperationalCommands {
    // Get the status of a specific operational agent
    Status {
        asset: Asset,
    },
    // Access the scheduling commands for an OperationalAgent (Technicial)
    Scheduling {
        #[clap(subcommand)]
        scheduling_commands: SchedulingCommands,
    },
    // Test the state of all operational agents
    Test,
}

#[derive(Subcommand, Debug)]
pub enum SchedulingCommands {
    // Get all the IDs of technicians
    OperationalIds {
        asset: Asset,
    },
    // Get information on a specific OperationalAgent
    OperationalAgent {
        asset: Asset,
        operational_id: String,
    },
}

impl OperationalCommands {
    pub fn execute(&self) -> SystemMessages {
        match self {
            OperationalCommands::Status { asset } => {
                let operational_request = OperationalRequest::AllOperationalStatus(asset.clone());

                SystemMessages::Operational(operational_request)
            }
            OperationalCommands::Scheduling {
                scheduling_commands,
            } => {
                match scheduling_commands {
                    SchedulingCommands::OperationalIds { asset } => {
                        let operational_request = OperationalRequest::GetIds(asset.clone());
                        SystemMessages::Operational(operational_request)
                    }
                    SchedulingCommands::OperationalAgent {
                        asset,
                        operational_id,
                    } => {
                        // TODO: Send message to the orchestrator to retrieve all information on a specific operational agent
                        let operational_request_scheduling =
                            OperationalSchedulingRequest::OperationalState(operational_id.clone());

                        let operational_request_message =
                            OperationalRequestMessage::Scheduling(operational_request_scheduling);

                        let operational_request = OperationalRequest::ForOperationalAgent((
                            asset.clone(),
                            operational_id.clone(),
                            operational_request_message,
                        ));
                        SystemMessages::Operational(operational_request)
                    }
                }
            }
            OperationalCommands::Test => {
                todo!();
                // let operational_request_message = OperationalRequestMessage::Test;

                // let operational_request = OperationalRequest::(operational_request_message);

                // SystemMessages::Operational(operational_request)
            }
        }
    }
}
