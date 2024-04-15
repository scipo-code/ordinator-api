pub mod supervisor_status_message;

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{agent_error::AgentError, resources::MainResources, Asset};

use self::supervisor_status_message::SupervisorStatusMessage;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupervisorRequest {
    pub asset: Asset,
    pub main_work_center: MainResources,
    pub supervisor_request_message: SupervisorRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorRequestMessage {
    Status(SupervisorStatusMessage),
    Test,
}

impl Message for SupervisorRequestMessage {
    type Result = Result<String, AgentError>;
}
