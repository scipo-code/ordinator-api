pub mod message_handlers;
pub mod requests;
pub mod responses;

use anyhow::Context;
use anyhow::Result;
use ordinator_scheduling_environment::Asset;
use requests::tactical_resources_message::TacticalResourceRequest;
use requests::tactical_scheduling_message::TacticalSchedulingRequest;
use requests::tactical_status_message::TacticalStatusMessage;
use requests::tactical_time_message::TacticalTimeRequest;
use responses::tactical_response_scheduling::TacticalResponseScheduling;
use responses::tactical_response_status::TacticalResponseStatus;
use responses::tactical_response_time::TacticalResponseTime;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalRequest
{
    pub asset: Asset,
    pub tactical_request_message: TacticalRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequestMessage
{
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingRequest),
    Resources(TacticalResourceRequest),
    Days(TacticalTimeRequest),
    Update,
}

#[derive(Serialize)]
pub struct TacticalResponse
{
    asset: Asset,
    tactical_response_message: TacticalResponseMessage,
}

impl TacticalResponse
{
    pub fn new(asset: Asset, tactical_response_message: TacticalResponseMessage) -> Self
    {
        Self {
            asset,
            tactical_response_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TacticalResponseMessage
{
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
