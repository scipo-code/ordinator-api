pub mod tactical_response_resources;
pub mod tactical_response_scheduling;
pub mod tactical_response_status;
pub mod tactical_response_time;
pub mod tactical_response_update;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseScheduling {}
use ordinator_scheduling_environment::time_environment::day::Day;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseStatus
{
    objective: u64,
    time_horizon: Vec<Day>,
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseTime {}
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TacticalResponseUpdate {}
