use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

use crate::strategic::TimePeriod;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestResource {
    // SetResources {
    //     resources: Vec<Resources>,
    //     period_imperium: Period,
    //     capacity: f64,
    // },
    GetLoadings {
        periods_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities {
        periods_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetPercentageLoadings {
        periods_end: String,
        resources: Option<Vec<Resources>>,
    },
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
