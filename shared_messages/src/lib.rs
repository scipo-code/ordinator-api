use actix::dev::MessageResponse;
use actix::prelude::*;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{self, Display};
pub mod resources;
use crate::resources::Resources;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "message_type")]
pub enum FrontendMessages {
    Strategic(StrategicRequests),
    Tactical,
    Worker,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub enum StrategicRequests {
    Status(StrategicStatusMessage),
    Scheduling(StrategicSchedulingMessage),
    Resources(StrategicResourcesMessage),
    Periods(PeriodsMessage),
}

// What should I call this message? It should not include the Strategic part as it is already part
// of the StrategicRequests enum. The frontend part is also wrong as the message is more general.
// We could choose to call it CLI or API message. Now for the Input, is this a valid name? I think
// that it is a very bad name for the what I am actually trying to achieve. Scheduling is better
// but there is still some redundancy in the name. Is the API actually a good name? I think that
// there must be something that serves the meaning of what I am trying to do better. I think that
// the name should go. Hmm SchedulingMessage is actually a good name.

impl Message for StrategicRequests {
    type Result = ();
}

#[derive(Debug)]
pub enum Response {
    Success(Option<String>),
    Failure,
}

impl Message for Response {
    type Result = ();
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Response::Success(string) => match string {
                Some(string) => string.clone(),
                None => "Command was successfully received and integrated".to_string(),
            },
            Response::Failure => {
                "Command was failed to be either received or integrated".to_string()
            }
        }
    }
}

impl<A, M> MessageResponse<A, M> for Response
where
    A: Actor,
    M: Message<Result = Response>,
{
    fn handle(
        self,
        ctx: &mut <A as Actor>::Context,
        msg: std::option::Option<actix::dev::OneshotSender<Response>>,
    ) {
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum StrategicStatusMessage {
    General,
    Period(String),
}

impl StrategicStatusMessage {
    pub fn new_period(period: String) -> Self {
        Self::Period(period)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicSchedulingMessage {
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>,
}

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

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicPeriodsMessage {
    pub period_lock: HashMap<String, bool>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimePeriod {
    pub period_string: String,
}

impl TimePeriod {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: u32,
    pub period_status: WorkOrderStatusInPeriod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderStatusInPeriod {
    pub locked_in_period: Option<TimePeriod>,
    #[serde(deserialize_with = "deserialize_period_set")]
    pub excluded_from_periods: HashSet<TimePeriod>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub struct PeriodsMessage {
    pub periods: Vec<u32>,
}

fn deserialize_period_set<'de, D>(deserializer: D) -> Result<HashSet<TimePeriod>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TimePeriod>::deserialize(deserializer)?;
    let mut set = HashSet::new();
    for time_period_map in vec {
        set.insert(TimePeriod {
            period_string: time_period_map.period_string,
        });
    }
    Ok(set)
}

impl Display for StrategicSchedulingMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\nWorkorder period mappings: {}, ",
            self.work_order_period_mappings.len(),
        )
    }
}

impl fmt::Display for StrategicRequests {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            StrategicRequests::Status(strategic_status_message) => {
                write!(f, "status")?;
                Ok(())
            }
            StrategicRequests::Scheduling(scheduling_message) => {
                for work_order_period_mapping in
                    scheduling_message.work_order_period_mappings.iter()
                {
                    writeln!(
                        f,
                        "work_order_period_mapping: {}",
                        work_order_period_mapping
                    )?;
                }

                Ok(())
            }
            StrategicRequests::Resources(resources_message) => {
                for manual_resource in resources_message.manual_resources.iter() {
                    writeln!(f, "manual_resource: {}", manual_resource)?;
                }
                Ok(())
            }
            StrategicRequests::Periods(period_message) => {
                write!(f, "period_message: {:?}", period_message)?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for WorkOrderPeriodMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "work_order: {}, period: {:?}",
            self.work_order_number, self.period_status
        )
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

impl WorkOrderPeriodMapping {
    pub fn new_test() -> Self {
        WorkOrderPeriodMapping {
            work_order_number: 2200002020,
            period_status: WorkOrderStatusInPeriod::new_test(),
        }
    }
}

impl WorkOrderStatusInPeriod {
    pub fn new_test() -> Self {
        let period_string = "2023-W47-48".to_string();
        WorkOrderStatusInPeriod {
            locked_in_period: Some(TimePeriod::new(period_string)),
            excluded_from_periods: HashSet::new(),
        }
    }
}
