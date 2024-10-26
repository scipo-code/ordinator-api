pub mod supervisor_response_resources;
pub mod supervisor_response_scheduling;
pub mod supervisor_response_status;
pub mod supervisor_response_time;
pub mod supervisor_scheduling_message;
pub mod supervisor_status_message;

use actix::Message;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use self::supervisor_response_resources::SupervisorResponseResources;
use self::supervisor_response_scheduling::SupervisorResponseScheduling;
use self::supervisor_response_status::SupervisorResponseStatus;
use self::supervisor_response_time::SupervisorResponseTime;
use self::supervisor_scheduling_message::SupervisorSchedulingMessage;
use self::supervisor_status_message::SupervisorStatusMessage;
use crate::{agent_error::AgentError, Asset};
use crate::{AlgorithmState, ConstraintState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupervisorRequest {
    pub asset: Asset,
    pub supervisor: SupervisorType,
    pub supervisor_request_message: SupervisorRequestMessage,
}

#[derive(ValueEnum, Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorType {
    Main,
    Other,
}

impl ToString for SupervisorType {
    fn to_string(&self) -> String {
        match self {
            SupervisorType::Main => "main".to_string(),
            SupervisorType::Other => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorRequestMessage {
    Status(SupervisorStatusMessage),
    Scheduling(SupervisorSchedulingMessage),
}

impl Message for SupervisorRequestMessage {
    type Result = Result<SupervisorResponseMessage, AgentError>;
}

#[derive(Serialize)]
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
