use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::Resources;

use super::TimePeriod;

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicResourcesMessage {
    manual_resources: HashMap<(Resources, String), f64>,
}

impl StrategicResourcesMessage {
    pub fn new(manual_resources: HashMap<(Resources, String), f64>) -> Self {
        Self { manual_resources }
    }

    pub fn get_manual_resources(&self) -> HashMap<(Resources, String), f64> {
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

impl StrategicResourcesMessage {
    pub fn new_test() -> Self {
        let mut manual_resources = HashMap::new();

        let period_string = "2023-W47-48".to_string();

        manual_resources.insert((Resources::MtnMech, period_string.clone()), 300.0);
        manual_resources.insert((Resources::MtnElec, period_string.clone()), 300.0);
        manual_resources.insert((Resources::Prodtech, period_string), 300.0);

        Self { manual_resources }
    }
}
