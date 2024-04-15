pub mod tactical_resources_message;
pub mod tactical_scheduling_message;
pub mod tactical_status_message;
pub mod tactical_time_message;

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{agent_error::AgentError, Asset};

use self::{
    tactical_resources_message::TacticalResourceMessage,
    tactical_scheduling_message::TacticalSchedulingMessage,
    tactical_status_message::TacticalStatusMessage, tactical_time_message::TacticalTimeMessage,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalRequest {
    pub asset: Asset,
    pub tactical_request_message: TacticalRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequestMessage {
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingMessage),
    Resources(TacticalResourceMessage),
    Days(TacticalTimeMessage),
    Test,
}

impl Message for TacticalRequestMessage {
    type Result = Result<String, AgentError>;
}
