pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::StrategicUserStatusCodes;
use serde::Deserialize;
use serde::Serialize;

use self::requests::*;
use self::responses::*;

pub type StrategicRequestMessage = RequestMessage<
    StrategicStatusMessage,
    StrategicRequestScheduling,
    StrategicRequestResource,
    StrategicTimeRequest,
    StrategicSchedulingEnvironmentCommands,
>;

// You should determine a better way of making this in the
// code I think that the best approach is to make something.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StrategicSchedulingEnvironmentCommands
{
    UserStatus(StrategicUserStatusCodes),
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum StrategicResponseMessage
{
    StateLink,
    Status(StrategicResponseStatus),
    Scheduling(StrategicResponseScheduling),
    Resources(StrategicResponseResources),
    Periods(StrategicResponsePeriods),
    Success,
}

// #[derive(Serialize)]
// pub struct StrategicInfeasibleCases {
//     pub respect_awsc: ConstraintState<String>,
//     pub respect_unloading: ConstraintState<String>,
//     pub respect_sch: ConstraintState<String>,
//     pub respect_aggregated_load: ConstraintState<String>,
// }

// impl Default for StrategicInfeasibleCases {
//     fn default() -> Self {
//         StrategicInfeasibleCases {
//             respect_awsc:
// ConstraintState::Infeasible("Infeasible".to_string()),
// respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
//             respect_sch:
// ConstraintState::Infeasible("Infeasible".to_string()),
// respect_aggregated_load:
// ConstraintState::Infeasible("Infeasible".to_string()),         }
//     }
// }
