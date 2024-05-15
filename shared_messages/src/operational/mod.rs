use actix::Message;
use serde::Serialize;

use crate::agent_error::AgentError;

use self::operational_response_status::OperationalResponseStatus;

pub mod operational_response_status;

pub enum OperationalRequestMessage {
    Status,
}

impl Message for OperationalRequestMessage {
    type Result = Result<OperationalResponseMessage, AgentError>;
}
pub enum OperationalResponseMessage {
    Status(OperationalResponseStatus),
}

#[derive(Serialize)]
pub enum OperationalResponse {
    Status,
}
