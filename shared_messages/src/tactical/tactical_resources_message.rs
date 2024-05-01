use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::worker_environment::resources::Resources;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceMessage {
    SetResources(HashMap<Resources, HashMap<String, f64>>),
    GetLoadings {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetPercentageLoadings {
        days_end: String,
        resources: Option<Vec<Resources>>,
    },
}

impl TacticalResourceMessage {
    pub fn new_set_resources(resources: HashMap<Resources, HashMap<String, f64>>) -> Self {
        TacticalResourceMessage::SetResources(resources)
    }
}
