pub mod message_handlers;
pub mod requests;
pub mod responses;

use std::fmt::Display;

use ordinator_actor_core::RequestMessage;
use ordinator_scheduling_environment::Asset;
use requests::SupervisorRequestResource;
use requests::SupervisorRequestScheduling;
use requests::SupervisorSchedulingEnvironmentCommands;
use requests::SupervisorSchedulingMessage;
use requests::SupervisorStatusMessage;
use requests::SupervisorTimeRequest;
use responses::SupervisorResponseResources;
use responses::SupervisorResponseScheduling;
use responses::SupervisorResponseStatus;
use responses::SupervisorResponseTime;
use serde::Deserialize;
use serde::Serialize;

pub type SupervisorRequestMessage = RequestMessage<
    SupervisorStatusMessage,
    SupervisorRequestScheduling,
    SupervisorRequestResource,
    SupervisorTimeRequest,
    SupervisorSchedulingEnvironmentCommands,
>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorType
{
    Main,
    Other,
}

#[derive(Serialize)]
pub enum SupervisorResponseMessage
{
    StateLink,
    Status(SupervisorResponseStatus),
    Scheduling(SupervisorResponseScheduling),
    Resources(SupervisorResponseResources),
    Time(SupervisorResponseTime),
    // Test(AlgorithmState<SupervisorInfeasibleCases>),
}

impl SupervisorResponseMessage
{
    pub fn status(self) -> SupervisorResponseStatus
    {
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
