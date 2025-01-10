pub mod strategic_request_periods_message;
pub mod strategic_request_resources_message;
pub mod strategic_request_scheduling_message;
pub mod strategic_request_status_message;

pub mod strategic_response_periods;
pub mod strategic_response_resources;
pub mod strategic_response_scheduling;
pub mod strategic_response_status;

use anyhow::Result;
use clap::Subcommand;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{self},
};

use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json_any_key::any_key_map;

use crate::{
    orchestrator::WorkOrdersStatus,
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{operation::Work, status_codes::StrategicUserStatusCodes},
        worker_environment::resources::Resources,
    },
    Asset, ConstraintState, LoadOperation,
};

use self::{
    strategic_request_periods_message::StrategicTimeRequest,
    strategic_request_resources_message::{ManualResource, StrategicResourceRequest},
    strategic_request_scheduling_message::StrategicSchedulingRequest,
    strategic_request_status_message::StrategicStatusMessage,
    strategic_response_periods::StrategicResponsePeriods,
    strategic_response_resources::StrategicResponseResources,
    strategic_response_scheduling::StrategicResponseScheduling,
    strategic_response_status::StrategicResponseStatus,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct StrategicObjectiveValue {
    pub objective_value: u64,
    pub urgency: (u64, u64),
    pub resource_penalty: (u64, u64),
    pub clustering_value: (u64, u64),
}

impl Default for StrategicObjectiveValue {
    fn default() -> Self {
        Self {
            objective_value: u64::MAX,
            urgency: (u64::MAX, u64::MAX),
            resource_penalty: (u64::MAX, u64::MAX),
            clustering_value: (u64::MAX, u64::MAX),
        }
    }
}

impl StrategicObjectiveValue {
    pub fn new(
        urgency: (u64, u64),
        resource_penalty: (u64, u64),
        clustering_value: (u64, u64),
    ) -> Self {
        Self {
            objective_value: 0,
            urgency,
            resource_penalty,
            clustering_value,
        }
    }

    pub fn aggregate_objectives(&mut self) {
        self.objective_value = self.urgency.0 * self.urgency.1
            + self.resource_penalty.0 * self.resource_penalty.1
            - self.clustering_value.0 * self.clustering_value.1;
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "strategic_message_type")]
pub struct StrategicRequest {
    pub asset: Asset,
    pub strategic_request_message: StrategicRequestMessage,
}

impl StrategicRequest {
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
}
#[derive(Subcommand, Serialize, Deserialize, Clone, Debug)]
pub enum StrategicSchedulingEnvironmentCommands {
    UserStatus(StrategicUserStatusCodes),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestMessage {
    Status(StrategicStatusMessage),
    Scheduling(StrategicSchedulingRequest),
    Resources(StrategicResourceRequest),
    Periods(StrategicTimeRequest),
    SchedulingEnvironment(StrategicSchedulingEnvironmentCommands),
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum StrategicResponseMessage {
    Status(StrategicResponseStatus),
    Scheduling(StrategicResponseScheduling),
    Resources(StrategicResponseResources),
    Periods(StrategicResponsePeriods),
    WorkOrder(WorkOrdersStatus),
    Success,
}
impl Message for StrategicRequestMessage {
    type Result = Result<StrategicResponseMessage>;
}

#[derive(Serialize)]
pub struct StrategicResponse {
    asset: Asset,
    strategic_response_message: StrategicResponseMessage,
}

impl StrategicResponse {
    pub fn new(asset: Asset, strategic_response_message: StrategicResponseMessage) -> Self {
        Self {
            asset,
            strategic_response_message,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimePeriod {
    pub period_string: String,
}

impl TimePeriod {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}
impl fmt::Display for StrategicRequestMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            StrategicRequestMessage::Status(strategic_status_message) => {
                write!(f, "status: {}", strategic_status_message)?;
                Ok(())
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                write!(f, "scheduling_message: {:?}", scheduling_message)?;

                Ok(())
            }
            StrategicRequestMessage::Resources(_resources_message) => {
                // for manual_resource in resources_message.get_manual_resources().iter() {
                //     writeln!(f, "manual_resource: {:?}", manual_resource)?;
                // }
                Ok(())
            }
            StrategicRequestMessage::Periods(period_message) => {
                write!(f, "period_message: {:?}", period_message)?;
                Ok(())
            }
            _ => todo!(),
        }
    }
}

impl fmt::Display for ManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "resource: {:?}, period: {}, capacity: {}",
            self.resource, self.period.period_string, self.capacity
        )
    }
}

impl TimePeriod {
    pub fn new(period_string: String) -> Self {
        Self { period_string }
    }
}

#[derive(Serialize)]
pub struct StrategicInfeasibleCases {
    pub respect_awsc: ConstraintState<String>,
    pub respect_unloading: ConstraintState<String>,
    pub respect_sch: ConstraintState<String>,
    pub respect_aggregated_load: ConstraintState<String>,
}

impl Default for StrategicInfeasibleCases {
    fn default() -> Self {
        StrategicInfeasibleCases {
            respect_awsc: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_sch: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_aggregated_load: ConstraintState::Infeasible("Infeasible".to_string()),
        }
    }
}

#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Debug, Clone)]
pub struct StrategicResources {
    #[serde(with = "any_key_map")]
    pub inner: HashMap<Resources, Periods>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Default, Debug, Clone)]
pub struct Periods(#[serde(with = "any_key_map")] pub HashMap<Period, Work>);

impl Periods {
    pub fn insert(&mut self, period: Period, load: Work) {
        self.0.insert(period, load);
    }
}

impl StrategicResources {
    pub fn new(resources: HashMap<Resources, Periods>) -> Self {
        Self { inner: resources }
    }

    pub fn update_load(
        &mut self,
        resource: &Resources,
        period: &Period,
        load: Work,
        load_operation: LoadOperation,
    ) {
        let resource_entry = self.inner.entry(resource.clone());
        let periods = match resource_entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Periods(HashMap::new())),
        };

        match periods.0.entry(period.clone()) {
            Entry::Occupied(mut entry) => match load_operation {
                LoadOperation::Add => *entry.get_mut() += load,
                LoadOperation::Sub => *entry.get_mut() -= load,
            },
            Entry::Vacant(entry) => match load_operation {
                LoadOperation::Add => {
                    entry.insert(load);
                }
                LoadOperation::Sub => {
                    entry.insert(load);
                }
            },
        };
    }

    pub fn update_resources(&mut self, resources: Self) {
        for resource in resources.inner {
            for period in resource.1 .0 {
                *self
                    .inner
                    .get_mut(&resource.0)
                    .unwrap()
                    .0
                    .get_mut(&period.0)
                    .unwrap() = period.1;
            }
        }
    }
}
