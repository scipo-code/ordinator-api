use serde::{Deserialize, Serialize};

use ordinator_scheduling_environment::time_environment::day::Day;

use crate::agents::tactical::TacticalObjectiveValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseStatus {
    objective: TacticalObjectiveValue,
    time_horizon: Vec<Day>,
}

impl TacticalResponseStatus {
    pub fn new(objective: TacticalObjectiveValue, time_horizon: Vec<Day>) -> Self {
        Self {
            objective,
            time_horizon,
        }
    }
}
