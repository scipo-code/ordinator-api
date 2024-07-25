use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::scheduling_environment::{time_environment::period::Period, worker_environment::resources::Resources};

use super::{Periods, StrategicResources, TimePeriod};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicResourceRequest {
    SetResources(StrategicResources),
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

impl StrategicResourceRequest {
    pub fn new_set_resources(manual_resources: StrategicResources) -> Self {
        Self::SetResources(manual_resources)
    }

    pub fn get_manual_resources(&self) -> Option<StrategicResources> {
        match self {
            Self::SetResources(manual_resource) => Some(manual_resource.clone()),
            _ => None,
        }
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

impl StrategicResourceRequest {
    pub fn new_test() -> Self {
        let mut manual_resources = HashMap::new();

        let period_string = Period::from_str("2023-W47-48").unwrap();

        let mut period_hash_map = Periods(HashMap::new());
        period_hash_map.insert(period_string, 300.0);

        manual_resources.insert(Resources::MtnMech, period_hash_map.clone());
        manual_resources.insert(Resources::MtnElec, period_hash_map.clone());
        manual_resources.insert(Resources::Prodtech, period_hash_map.clone());

        Self::SetResources(StrategicResources {
            inner: manual_resources,
        })
    }
}
