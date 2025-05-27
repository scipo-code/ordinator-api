pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use requests::TacticalRequestResource;
use requests::TacticalRequestScheduling;
use requests::TacticalSchedulingEnvironmentCommands;
use requests::TacticalStatusMessage;
use requests::TacticalTimeRequest;
use responses::TacticalResponseScheduling;
use responses::TacticalResponseStatus;
use responses::TacticalResponseTime;
use serde::Serialize;

pub type TacticalRequestMessage = RequestMessage<
    TacticalStatusMessage,
    TacticalRequestScheduling,
    TacticalRequestResource,
    TacticalTimeRequest,
    TacticalSchedulingEnvironmentCommands,
>;

#[derive(Debug, Serialize)]
pub enum TacticalResponseMessage
{
    FreeStringResponse(String),
    Status(TacticalResponseStatus),
    Scheduling(TacticalResponseScheduling),
    // Resources(TacticalResourceResponse),
    Time(TacticalResponseTime),
    Update,
}

// TODO [ ]
// Consider reintroducing this into the code at a later stage the idea is good.
// #[derive(Debug, Clone, Serialize)]
// pub struct TacticalInfeasibleCases {
//     pub aggregated_load: ConstraintState<String>,
//     pub earliest_start_day: ConstraintState<String>,
//     pub all_scheduled: ConstraintState<String>,
//     pub respect_period_id: ConstraintState<String>,
// }

// impl Default for TacticalInfeasibleCases {
//     fn default() -> Self {
//         TacticalInfeasibleCases {
//             aggregated_load:
// ConstraintState::Infeasible("Infeasible".to_owned()),
// earliest_start_day: ConstraintState::Infeasible("Infeasible".to_owned()),
//             all_scheduled:
// ConstraintState::Infeasible("Infeasible".to_owned()),
// respect_period_id: ConstraintState::Infeasible("Infeasible".to_owned()),
//         }
//     }
// }
