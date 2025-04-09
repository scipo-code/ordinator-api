use ordinator_scheduling_environment::time_environment::day::Day;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseScheduling {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseStatus
{
    objective: u64,
    time_horizon: Vec<Day>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseTime {}

#[derive(Debug, Serialize)]
pub struct TacticalResponseUpdate {}
