pub mod supervisor_response_resources;
pub mod supervisor_response_scheduling;
pub mod supervisor_response_status;
pub mod supervisor_response_time;
pub mod supervisor_status_message;

use actix::Message;
use serde::{Deserialize, Serialize};

use self::supervisor_response_resources::SupervisorResponseResources;
use self::supervisor_response_scheduling::SupervisorResponseScheduling;
use self::supervisor_response_status::SupervisorResponseStatus;
use self::supervisor_response_time::SupervisorResponseTime;
use self::supervisor_status_message::SupervisorStatusMessage;
use crate::models::worker_environment::resources::MainResources;
use crate::{agent_error::AgentError, Asset};
use crate::{AlgorithmState, ConstraintState};

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
    type Result = Result<SupervisorResponseMessage, AgentError>;
}

pub struct SupervisorResponse {
    asset: Asset,
    supervisor_response_message: SupervisorResponseMessage,
}

impl SupervisorResponse {
    pub fn new(asset: Asset, supervisor_response_message: SupervisorResponseMessage) -> Self {
        Self {
            asset,
            supervisor_response_message,
        }
    }
}

#[derive(Serialize)]
pub enum SupervisorResponseMessage {
    Status(SupervisorResponseStatus),
    Scheduling(SupervisorResponseScheduling),
    Resources(SupervisorResponseResources),
    Time(SupervisorResponseTime),
    Test(AlgorithmState<SupervisorInfeasibleCases>),
}

impl SupervisorResponseMessage {
    pub fn status(self) -> SupervisorResponseStatus {
        match self {
            Self::Status(supervisor_response_status) => supervisor_response_status,
            _ => panic!("The underlying variant of the enum was not a status response"),
        }
    }
}

#[derive(Serialize)]
pub struct SupervisorInfeasibleCases {
    pub respect_main_work_center: ConstraintState<String>,
}

impl Default for SupervisorInfeasibleCases {
    fn default() -> Self {
        Self {
            respect_main_work_center: ConstraintState::Infeasible("Infeasible".to_string()),
        }
    }
}
