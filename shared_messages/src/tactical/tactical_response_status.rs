use serde::{Deserialize, Serialize};

use crate::scheduling_environment::time_environment::period::Period;

#[derive(Debug, Serialize, Deserialize)]
pub struct TacticalResponseStatus {
    id: i32,
    objective: f64,
    time_horizon: Vec<Period>,
}

impl TacticalResponseStatus {
    pub fn new(id: i32, objective: f64, time_horizon: Vec<Period>) -> Self {
        Self {
            id,
            objective,
            time_horizon,
        }
    }
}
