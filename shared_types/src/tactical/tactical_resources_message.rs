use serde::{Deserialize, Serialize};

use crate::scheduling_environment::worker_environment::resources::Resources;

use super::TacticalResources;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceRequest {
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

impl TacticalResourceRequest {
    pub fn new_set_resources(resources: TacticalResources) -> Self {
        TacticalResourceRequest::SetResources(resources)
    }
}
