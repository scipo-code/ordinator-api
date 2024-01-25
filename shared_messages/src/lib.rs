use actix::dev::MessageResponse;
use actix::prelude::*;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{self, Display};
pub mod resources;
use crate::resources::Resources;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "message_type")]
pub enum FrontendMessages {
    Scheduler(SchedulerRequests),
    WorkPlanner,
    Worker,
    Activity,
    WorkCenter,
    WorkOrder,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub enum SchedulerRequests {
    Input(FrontendInputSchedulerMessage),
    Period(FrontendUpdatePeriod),
    WorkPlanner(WorkPlannerMessage),
    GetWorkerNumber,
}

impl Message for SchedulerRequests {
    type Result = ();
}

#[derive(Debug)]
pub enum Response {
    Success,
    Failure,
}

impl Message for Response {
    type Result = ();
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Response::Success => "Command was successfully received and integrated".to_string(),
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

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkPlannerMessage {
    cannot_schedule: Vec<u32>,
    under_loaded_work_centers: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FrontendInputSchedulerMessage {
    pub name: String,
    pub platform: String,
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>,
    pub manual_resources: Vec<ManualResource>,
    pub period_lock: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManualResource {
    pub resource: Resources,
    pub period: TimePeriod,
    pub capacity: f64,
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
pub struct FrontendUpdatePeriod {
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

impl Display for FrontendInputSchedulerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}, 
            \nPlatform: {}, 
            \nWorkorder period mappings: {}, 
            \nManual Resource: {},
            \nPeriod Lock: {:?}",
            self.name,
            self.platform,
            self.work_order_period_mappings.len(),
            self.manual_resources.len(),
            self.period_lock
        )
    }
}

impl fmt::Display for SchedulerRequests {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            SchedulerRequests::Input(input) => {
                write!(f, "name: {}", input.name)?;

                for work_order_period_mapping in input.work_order_period_mappings.iter() {
                    writeln!(
                        f,
                        "work_order_period_mapping: {}",
                        work_order_period_mapping
                    )?;
                }
                for manual_resource in input.manual_resources.iter() {
                    writeln!(f, "manual_resource: {}", manual_resource)?;
                }
                Ok(())
            }
            SchedulerRequests::WorkPlanner(work_planner) => {
                write!(f, "work_planner: {:?}", work_planner.cannot_schedule)?;
                Ok(())
            }
            SchedulerRequests::Period(period) => {
                write!(f, "period: {:?}", period)?;
                Ok(())
            }
            SchedulerRequests::GetWorkerNumber => {
                write!(f, "get_worker_number")?;
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
