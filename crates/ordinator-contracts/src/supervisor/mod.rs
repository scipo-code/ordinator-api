pub mod requests;
pub mod responses;

use std::fmt::Display;

use clap::ValueEnum;
use ordinator_scheduling_environment::Asset;
use requests::supervisor_scheduling_message::SupervisorSchedulingMessage;
use requests::supervisor_status_message::SupervisorStatusMessage;
use responses::supervisor_response_resources::SupervisorResponseResources;
use responses::supervisor_response_scheduling::SupervisorResponseScheduling;
use responses::supervisor_response_status::SupervisorResponseStatus;
use responses::supervisor_response_time::SupervisorResponseTime;
use serde::Deserialize;
use serde::Serialize;

pub type SupervisorObjectiveValue = u64;

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

impl Display for SupervisorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupervisorType::Main => write!(f, "main"),
            SupervisorType::Other => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorRequestMessage {
    Status(SupervisorStatusMessage),
    Scheduling(SupervisorSchedulingMessage),
    Update,
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
    // Test(AlgorithmState<SupervisorInfeasibleCases>),
}

impl SupervisorResponseMessage {
    pub fn status(self) -> SupervisorResponseStatus {
        match self {
            Self::Status(supervisor_response_status) => supervisor_response_status,
            _ => panic!("The underlying variant of the enum was not a status response"),
        }
    }
}

// #[derive(Serialize)]
// pub struct SupervisorInfeasibleCases {
//     pub respect_main_work_center: ConstraintState<String>,
// }

// impl Default for SupervisorInfeasibleCases {
//     fn default() -> Self {
//         Self {
//             respect_main_work_center:
// ConstraintState::Infeasible("Infeasible".to_string()),         }
//     }
// }
