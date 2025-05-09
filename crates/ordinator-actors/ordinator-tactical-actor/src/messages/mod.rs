pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_scheduling_environment::Asset;
use requests::TacticalResourceRequest;
use requests::TacticalSchedulingRequest;
use requests::TacticalStatusMessage;
use requests::TacticalTimeRequest;
use responses::TacticalResponseScheduling;
use responses::TacticalResponseStatus;
use responses::TacticalResponseTime;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalRequest {
    pub asset: Asset,
    pub tactical_request_message: TacticalRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequestMessage {
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingRequest),
    Resources(TacticalResourceRequest),
    Days(TacticalTimeRequest),
    Update,
}

#[derive(Serialize)]
pub struct TacticalResponse {
    asset: Asset,
    tactical_response_message: TacticalResponseMessage,
}

impl TacticalResponse {
    pub fn new(asset: Asset, tactical_response_message: TacticalResponseMessage) -> Self {
        Self {
            asset,
            tactical_response_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TacticalResponseMessage {
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
