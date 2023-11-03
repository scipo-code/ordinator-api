use actix::prelude::*;
use serde::{Serialize, Deserialize, Deserializer, de::Error};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use crate::models::period::Period;

#[derive(Serialize, Deserialize)]
#[serde(tag = "scheduler_message_type")]
#[derive(Debug)]
pub enum SchedulerRequests {
    Input(FrontendInputSchedulerMessage),
    WorkPlanner(WorkPlannerMessage),
    ExecuteIteration,
}

#[derive(Serialize, Deserialize)]
pub struct InputSchedulerMessage {
    pub name: String,
    pub platform: String,
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>, // For each work order only one of these can be true
    pub manual_resources: HashMap<(String, String), f64>,
    pub period_lock: HashMap<String, bool>
}

impl InputSchedulerMessage {
    pub fn get_manual_resources(&self) -> HashMap<(String, String), f64> {
        self.manual_resources.clone()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetAgentAddrMessage<T: actix::Actor> {
    pub addr: Addr<T>
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct WorkPlannerMessage {
    cannot_schedule: Vec<u32>,
    under_loaded_work_centers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ManualResource {
    resource: String,
    period: TimePeriod,
    capacity: f64
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct TimePeriod {
    period_string: String,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct FrontendInputSchedulerMessage {
    name: String,
    platform: String,
    work_order_period_mappings: Vec<WorkOrderPeriodMapping>,
    manual_resources: Vec<ManualResource>,
    period_lock: HashMap<String, bool>
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: u32,
    pub period_status: WorkOrderStatusInPeriod,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct WorkOrderStatusInPeriod {
    #[serde(deserialize_with = "deserialize_period_option")]
    pub locked_in_period: Option<Period>,
    #[serde(deserialize_with = "deserialize_period_set")]
    pub excluded_from_periods: HashSet<Period>,
}

struct SchedulerResources<'a>(&'a HashMap<(String, String), f64>);

impl Display for SchedulerResources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "--------------------------")?;
        for ((resource, period), capacity) in self.0 {
            writeln!(f, "Resource: {}\nPeriod: {}\nCapacity: {}", resource, period, capacity)?;
        }
        write!(f, "--------------------------")
    }
}

impl Message for SchedulerRequests {
    type Result = ();
}

impl Display for InputSchedulerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let manual_resources_pretty = SchedulerResources(&self.manual_resources);
        write!(f, 
            "Name: {}, 
            \nPlatform: {}, 
            \nSchedule Work Order: {}, 
            \nManual Resource: {},
            \nPeriod Lock: {:?}", 
            self.name, 
            self.platform, 
            self.work_order_period_mappings.len(), 
            manual_resources_pretty,
            self.period_lock
        )
    } 
}

impl From<FrontendInputSchedulerMessage> for InputSchedulerMessage {
    fn from(raw: FrontendInputSchedulerMessage) -> Self {
        let mut manual_resources_map: HashMap<(String, String), f64> = HashMap::new();
        for res in raw.manual_resources {
            manual_resources_map.insert((res.resource, res.period.period_string), res.capacity);   
        }
        println!("{:?}", manual_resources_map);
    


        InputSchedulerMessage {
            name: raw.name,
            platform: raw.platform,
            work_order_period_mappings: raw.work_order_period_mappings,
            manual_resources: manual_resources_map,
            period_lock: raw.period_lock
        }
    }
}

impl Display for FrontendInputSchedulerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
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

fn deserialize_period_option<'de, D>(deserializer: D) -> Result<Option<Period>, D::Error>
where
    D: Deserializer<'de>,
{
    let option = Option::<TimePeriod>::deserialize(deserializer)?;
    match option {
        Some(time_period_map) => Period::new_from_string(&time_period_map.period_string)
            .map(Some)
            .map_err(Error::custom),
        None => Ok(None),
    }
}

fn deserialize_period_set<'de, D>(deserializer: D) -> Result<HashSet<Period>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TimePeriod>::deserialize(deserializer)?;
    let mut set = HashSet::new();
    for time_period_map in vec {
        let period = Period::new_from_string(&time_period_map.period_string).map_err(Error::custom)?;
        set.insert(period);
    }
    Ok(set)
}