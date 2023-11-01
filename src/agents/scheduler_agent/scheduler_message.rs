use actix::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

#[derive(Serialize, Deserialize)]
#[serde(tag = "scheduler_message_type")]
pub enum SchedulerMessages {
    Input(RawInputMessage),
    WorkPlanner(WorkPlannerMessage),
    ExecuteIteration,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct InputMessage {
    name: String,
    platform: String,
    schedule_work_order: Vec<OrderPeriod>,
    unschedule_work_order: HashSet<u32>,
    manual_resources: HashMap<(String, Period), f64>,
    period_lock: HashMap<String, bool>
}

impl InputMessage {
    pub fn get_manual_resources(&self) -> HashMap<(String, Period), f64> {
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
pub struct ManualResource {
    resource: String,
    period: String,
    capacity: f64
}

#[derive(Serialize, Deserialize)]
pub struct RawInputMessage {
    name: String,
    platform: String,
    schedule_work_order: Vec<OrderPeriod>,
    unschedule_work_order: HashSet<u32>,
    manual_resources: Vec<ManualResource>,
    period_lock: HashMap<String, bool>
}

struct SchedulerResources<'a>(&'a HashMap<(String, Period), f64>);

impl Display for SchedulerResources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "--------------------------")?;
        for ((resource, period), capacity) in self.0 {
            writeln!(f, "Resource: {}\nPeriod: {}\nCapacity: {}", resource, period, capacity)?;
        }
        write!(f, "--------------------------")
    }
}

impl Message for SchedulerMessages {
    type Result = ();
}

impl Display for InputMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let manual_resources_pretty = SchedulerResources(&self.manual_resources);
        write!(f, 
            "Name: {}, 
            \nPlatform: {}, 
            \nSchedule Work Order: {}, 
            \nUnschedule Work Order: {:?}, 
            \nManual Resource: {},
            \nPeriod Lock: {:?}", 
            self.name, 
            self.platform, 
            self.schedule_work_order.len(), 
            self.unschedule_work_order, 
            manual_resources_pretty,
            self.period_lock
        )
    } 
}

impl From<RawInputMessage> for InputMessage {
    fn from(raw: RawInputMessage) -> Self {
        let mut manual_resources_map: HashMap<(String, Period), f64> = HashMap::new();
        for res in raw.manual_resources {
            let period = Period::new_from_string(&res.period).expect(format!("could not parse period. File: {}, line: {}", file!(), line!()).as_str() );
            manual_resources_map.insert((res.resource, period), res.capacity);   
        }
        println!("{:?}", manual_resources_map);
    
        InputMessage {
            name: raw.name,
            platform: raw.platform,
            schedule_work_order: raw.schedule_work_order,
            unschedule_work_order: raw.unschedule_work_order,
            manual_resources: manual_resources_map,
            period_lock: raw.period_lock
        }
    }
}

impl Display for RawInputMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "Name: {}, 
            \nPlatform: {}, 
            \nSchedule Work Order: {}, 
            \nUnschedule Work Order: {:?}, 
            \nnManual Resource: {},
            \nPeriod Lock: {:?}", 
            self.name, 
            self.platform, 
            self.schedule_work_order.len(), 
            self.unschedule_work_order, 
            self.manual_resources.len(),
            self.period_lock
        )
    } 
}