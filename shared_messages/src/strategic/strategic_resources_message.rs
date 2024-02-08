use serde::{Deserialize, Serialize};

use crate::resources::Resources;

use super::TimePeriod;

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicResourcesMessage {
    manual_resources: Vec<ManualResource>,
}

impl StrategicResourcesMessage {
    pub fn new(manual_resources: Vec<ManualResource>) -> Self {
        Self { manual_resources }
    }

    pub fn get_manual_resources(&self) -> Vec<ManualResource> {
        self.manual_resources.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManualResource {
    pub resource: Resources,
    pub period: TimePeriod,
    pub capacity: f64,
}

impl ManualResource {
    pub fn new(resource: Resources, period: TimePeriod, capacity: f64) -> Self {
        Self {
            resource,
            period,
            capacity,
        }
    }
}
