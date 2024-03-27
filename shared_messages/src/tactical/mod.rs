pub mod tactical_resources_message;
pub mod tactical_scheduling_message;
pub mod tactical_status_message;
pub mod tactical_time_message;

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::agent_error::AgentError;

use self::{
    tactical_resources_message::TacticalResourceMessage,
    tactical_scheduling_message::TacticalSchedulingMessage,
    tactical_status_message::TacticalStatusMessage, tactical_time_message::TacticalTimeMessage,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequest {
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingMessage),
    Resources(TacticalResourceMessage),
    Days(TacticalTimeMessage),
}

impl Message for TacticalRequest {
    type Result = Result<String, AgentError>;
}
