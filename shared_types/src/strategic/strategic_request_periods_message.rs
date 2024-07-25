use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "scheduler_message_type")]
pub struct StrategicTimeRequest {
    pub periods: Vec<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicPeriodsMessage {
    pub period_lock: HashMap<String, bool>,
}
