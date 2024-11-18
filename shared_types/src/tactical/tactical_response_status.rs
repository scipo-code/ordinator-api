use serde::{Deserialize, Serialize};

use crate::scheduling_environment::time_environment::day::Day;

use super::TacticalObjectiveValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseStatus {
    id: i32,
    objective: TacticalObjectiveValue,
    time_horizon: Vec<Day>,
}

impl TacticalResponseStatus {
    pub fn new(id: i32, objective: TacticalObjectiveValue, time_horizon: Vec<Day>) -> Self {
        Self {
            id,
            objective,
            time_horizon,
        }
    }
}
