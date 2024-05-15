use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{time_environment::day::Day, worker_environment::resources::Resources};

use super::{Days, TacticalResources};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceMessage {
    SetResources(TacticalResources),
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
    pub fn new_set_resources(resources: TacticalResources) -> Self {
        TacticalResourceMessage::SetResources(resources)
    }
}
