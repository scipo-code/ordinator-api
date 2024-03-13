use clap::Subcommand;
use shared_messages::{tactical::TacticalRequest, SystemMessages};

#[derive(Subcommand, Debug)]
pub enum TacticalCommands {
    /// Get the status of the tactical agent
    Status,
    /// Get the objectives of the tactical agent
    Objectives,
}

impl TacticalCommands {
    pub fn execute(&self) -> shared_messages::SystemMessages {
        match self {
            TacticalCommands::Status => {
                dbg!("TacticalAgent Status Message");
                SystemMessages::Tactical(TacticalRequest::Status)
            }
            TacticalCommands::Objectives => {
                todo!()
            }
        }
    }
}
