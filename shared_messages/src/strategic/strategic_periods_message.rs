use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub struct PeriodsMessage {
    pub periods: Vec<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicPeriodsMessage {
    pub period_lock: HashMap<String, bool>,
}
