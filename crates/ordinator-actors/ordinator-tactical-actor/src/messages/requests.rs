pub mod tactical_resources_message;
pub mod tactical_scheduling_message;
pub mod tactical_status_message;
pub mod tactical_time_message;
pub mod tactical_update_message;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

// This should be a set of HTTP GET and POST endpoints. That is crucial to
// understand here. The goal here is to have an optimal backend data structure
// and then have a JSON api data structure. That is the best way of implementing
// this I do not see a different way.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceRequest
{
    // SetResources(TacticalResources),
    GetLoadings
    {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities
    {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetPercentageLoadings
    {
        days_end: String,
        resources: Option<Vec<Resources>>,
    },
}
use serde::Deserialize;
use serde::Serialize;

use crate::strategic::requests::strategic_request_scheduling_message::ScheduleChange;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalSchedulingRequest
{
    Schedule(ScheduleChange),
    ScheduleMultiple(Vec<ScheduleChange>),
    ExcludeFromDay(ScheduleChange),
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalStatusMessage
{
    General,
    Day(String),
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum TacticalTimeRequest
{
    Days,
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalUpdateRequest {}
