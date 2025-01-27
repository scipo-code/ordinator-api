pub mod tactical_resources_message;
pub mod tactical_response_resources;
pub mod tactical_response_scheduling;
pub mod tactical_response_status;
pub mod tactical_response_time;
pub mod tactical_response_update;
pub mod tactical_scheduling_message;
pub mod tactical_status_message;
pub mod tactical_time_message;
pub mod tactical_update_message;

use std::collections::HashMap;

use crate::{
    scheduling_environment::{
        time_environment::day::Day, work_order::operation::Work,
        worker_environment::resources::Resources,
    },
    Asset, ConstraintState,
};
use actix::Message;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;

use self::{
    tactical_resources_message::TacticalResourceRequest,
    tactical_response_resources::TacticalResponseResources,
    tactical_response_scheduling::TacticalResponseScheduling,
    tactical_response_status::TacticalResponseStatus, tactical_response_time::TacticalResponseTime,
    tactical_scheduling_message::TacticalSchedulingRequest,
    tactical_status_message::TacticalStatusMessage, tactical_time_message::TacticalTimeRequest,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct TacticalObjectiveValue(pub u64);

impl Default for TacticalObjectiveValue {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalRequest {
    pub asset: Asset,
    pub tactical_request_message: TacticalRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequestMessage {
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingRequest),
    Resources(TacticalResourceRequest),
    Days(TacticalTimeRequest),
    Update,
}

impl Message for TacticalRequestMessage {
    type Result = Result<TacticalResponseMessage>;
}

#[derive(Serialize)]
pub struct TacticalResponse {
    asset: Asset,
    tactical_response_message: TacticalResponseMessage,
}

impl TacticalResponse {
    pub fn new(asset: Asset, tactical_response_message: TacticalResponseMessage) -> Self {
        Self {
            asset,
            tactical_response_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TacticalResponseMessage {
    Status(TacticalResponseStatus),
    Scheduling(TacticalResponseScheduling),
    Resources(TacticalResponseResources),
    Time(TacticalResponseTime),
    Update,
}

#[derive(Debug, Clone, Serialize)]
pub struct TacticalInfeasibleCases {
    pub aggregated_load: ConstraintState<String>,
    pub earliest_start_day: ConstraintState<String>,
    pub all_scheduled: ConstraintState<String>,
    pub respect_period_id: ConstraintState<String>,
}

impl Default for TacticalInfeasibleCases {
    fn default() -> Self {
        TacticalInfeasibleCases {
            aggregated_load: ConstraintState::Infeasible("Infeasible".to_owned()),
            earliest_start_day: ConstraintState::Infeasible("Infeasible".to_owned()),
            all_scheduled: ConstraintState::Infeasible("Infeasible".to_owned()),
            respect_period_id: ConstraintState::Infeasible("Infeasible".to_owned()),
        }
    }
}
#[derive(Eq, PartialEq, Default, Serialize, Deserialize, Debug, Clone)]
pub struct TacticalResources {
    #[serde(with = "any_key_map")]
    pub resources: HashMap<Resources, Days>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct Days {
    #[serde(with = "any_key_map")]
    pub days: HashMap<Day, Work>,
}

impl Days {
    pub fn new(days: HashMap<Day, Work>) -> Self {
        Self { days }
    }

    pub fn get(&self, day: &Day) -> &Work {
        self.days.get(day).unwrap()
    }

    pub fn day_mut(&mut self, day: &Day) -> &mut Work {
        self.days.get_mut(day).unwrap()
    }
}

impl TacticalResources {
    pub fn new(resources: HashMap<Resources, Days>) -> Self {
        TacticalResources { resources }
    }

    pub fn get_resource(&self, resource: &Resources, day: &Day) -> &Work {
        self.resources.get(resource).unwrap().get(day)
    }

    pub fn new_from_data(resources: Vec<Resources>, tactical_days: Vec<Day>, load: Work) -> Self {
        let mut resource_capacity: HashMap<Resources, Days> = HashMap::new();
        for resource in resources {
            let mut days = HashMap::new();
            for day in tactical_days.iter() {
                days.insert(day.clone(), load);
            }

            resource_capacity.insert(resource, Days { days });
        }
        TacticalResources::new(resource_capacity)
    }

    pub fn update_resources(&mut self, resources: Self) {
        for resource in resources.resources {
            for day in resource.1.days {
                *self
                    .resources
                    .get_mut(&resource.0)
                    .unwrap()
                    .days
                    .get_mut(&day.0)
                    .unwrap() = day.1;
            }
        }
    }

    pub fn determine_period_load(
        &self,
        resource: &Resources,
        period: &crate::scheduling_environment::time_environment::period::Period,
    ) -> Result<Work> {
        let days = &self
            .resources
            .get(resource)
            .with_context(|| "The resources between the strategic and the tactical should always correspond, unless that the tactical has not been initialized yet".to_string())?
            .days;

        Ok(days
            .iter()
            .filter(|(day, _)| period.contains_date(day.date().date_naive()))
            .map(|(_, work)| work)
            .fold(Work::from(0.0), |acc, work| &acc + work))
    }
}
