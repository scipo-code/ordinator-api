pub mod strategic_request_periods_message;
pub mod strategic_request_resources_message;
pub mod strategic_request_scheduling_message;
pub mod strategic_request_status_message;

pub mod strategic_response_periods;
pub mod strategic_response_resources;
pub mod strategic_response_scheduling;
pub mod strategic_response_status;

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Write},
};

use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json_any_key::any_key_map;

use crate::{
    agent_error::AgentError,
    models::{time_environment::period::Period, worker_environment::resources::Resources},
    AlgorithmState, Asset, ConstraintState, LoadOperation,
};

use self::{
    strategic_request_periods_message::StrategicTimeMessage,
    strategic_request_resources_message::{ManualResource, StrategicResourceMessage},
    strategic_request_scheduling_message::{StrategicSchedulingMessage, WorkOrderStatusInPeriod},
    strategic_request_status_message::StrategicStatusMessage,
    strategic_response_periods::StrategicResponsePeriods,
    strategic_response_resources::StrategicResponseResources,
    strategic_response_scheduling::StrategicResponseScheduling,
    strategic_response_status::{StrategicResponseStatus, WorkOrdersStatus},
};

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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestMessage {
    Status(StrategicStatusMessage),
    Scheduling(StrategicSchedulingMessage),
    Resources(StrategicResourceMessage),
    Periods(StrategicTimeMessage),
    Test,
}

impl Message for StrategicRequestMessage {
    type Result = Result<StrategicResponseMessage, AgentError>;
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

#[derive(Serialize)]
pub enum StrategicResponseMessage {
    Status(StrategicResponseStatus),
    Scheduling(StrategicResponseScheduling),
    Resources(StrategicResponseResources),
    Periods(StrategicResponsePeriods),
    WorkOrder(WorkOrdersStatus),
    Test(AlgorithmState<StrategicInfeasibleCases>),
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
            StrategicRequestMessage::Resources(resources_message) => {
                for manual_resource in resources_message.get_manual_resources().iter() {
                    writeln!(f, "manual_resource: {:?}", manual_resource)?;
                }
                Ok(())
            }
            StrategicRequestMessage::Periods(period_message) => {
                write!(f, "period_message: {:?}", period_message)?;
                Ok(())
            }
            StrategicRequestMessage::Test => {
                write!(f, "test")?;
                Ok(())
            }
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
#[derive(Default, Serialize, Debug, Clone)]
pub struct StrategicResources {
    #[serde(with = "any_key_map")]
    pub inner: HashMap<Resources, Periods>,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct Periods(pub HashMap<Period, f64>);

impl Periods {
    pub fn insert(&mut self, period: Period, load: f64) {
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
        load: f64,
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

    pub fn to_string(&self, number_of_periods: u32) -> String {
        let mut string = String::new();
        let mut periods = self
            .inner
            .values()
            .flat_map(|inner_map| inner_map.0.keys())
            .collect::<Vec<_>>();
        periods.sort();
        periods.dedup();

        write!(string, "{:<12}", "Resource").ok();
        for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
            if nr_period == 0 {
                write!(string, "{:>12}", period.period_string()).ok();
            } else if nr_period == 1 || nr_period == 2 {
                write!(string, "{:>12}", period.period_string()).ok();
            } else {
                write!(string, "{:>12}", period.period_string()).ok();
            }
        }
        writeln!(string).ok();

        let mut sorted_resources: Vec<&Resources> = self.inner.keys().collect();

        sorted_resources
            .sort_by(|resource_a, resource_b| resource_a.to_string().cmp(&resource_b.to_string()));
        for resource in sorted_resources {
            let inner_map = self.inner.get(resource).unwrap();
            write!(string, "{:<12}", resource.variant_name()).unwrap();
            for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
                let value = inner_map.0.get(period).unwrap_or(&0.0);
                if nr_period == 0 {
                    write!(string, "{:>12}", value.round().to_string()).ok();
                } else if nr_period == 1 || nr_period == 2 {
                    write!(string, "{:>12}", value.round().to_string()).ok();
                } else {
                    write!(string, "{:>12}", value.round()).ok();
                }
            }
            writeln!(string).ok();
        }
        string
    }
}
