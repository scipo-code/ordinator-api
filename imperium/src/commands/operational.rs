use clap::Subcommand;
use shared_types::{
    operational::{
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_request_status::OperationalStatusRequest, OperationalRequest,
        OperationalRequestMessage, OperationalTarget,
    },
    SystemMessages,
};

#[derive(Subcommand, Debug)]
pub enum OperationalCommands {
    // Get the status of a specific operational agent
    Status {
        operational_target: OperationalTarget,
    },
    // Access the scheduling commands for an OperationalAgent (Technicial)
    Scheduling {
        operational_target: OperationalTarget,
    },
    // Test the state of all operational agents
    Test,
}

impl OperationalCommands {
    pub fn execute(&self) -> SystemMessages {
        match self {
            OperationalCommands::Status { operational_target } => {
                let operational_request_status = OperationalStatusRequest::General;

                let operational_request_message =
                    OperationalRequestMessage::Status(operational_request_status);

                let operational_request = OperationalRequest::new(
                    operational_target.clone(),
                    operational_request_message,
                );

                SystemMessages::Operational(operational_request)
            }
            OperationalCommands::Scheduling { operational_target } => {
                let operational_request_scheduling = OperationalSchedulingRequest::ListEvents;

                let operational_request_message =
                    OperationalRequestMessage::Scheduling(operational_request_scheduling);

                let operational_request = OperationalRequest::new(
                    operational_target.clone(),
                    operational_request_message,
                );
                SystemMessages::Operational(operational_request)
            }
            OperationalCommands::Test => {
                let operational_request_message = OperationalRequestMessage::Test;

                let operational_request =
                    OperationalRequest::new(OperationalTarget::All, operational_request_message);

                SystemMessages::Operational(operational_request)
            }
        }
    }
}
