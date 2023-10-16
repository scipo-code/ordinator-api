use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::models::order_period::OrderPeriod;
use std::fmt::{self, Display};
use crate::models::period::Period;

/// Represents various types of messages that can be sent to the scheduler.
/// This ensures standardized communication with the rest of the system 
/// for business compliance.
enum SchedulerMessage {
    Input(InputMessage),
    Output(OutputMessage),
    WorkPlanner(WorkPlannerMessage),
}

/// Represents the message received from the front-end.
/// The data from the front-end will be used to instantiate this struct.
#[derive(Serialize, Deserialize)]
pub struct InputMessage {
    message_type: String, 
    name: String,
    platform: String,
    schedule_work_order: Vec<OrderPeriod>,
    unschedule_work_order: HashSet<u32>,
    manual_resource: HashMap<(String, Period), f64>,
    period_lock: HashMap<String, bool>
}

/// Represents the message sent to the front-end.
#[allow(dead_code)]
struct OutputMessage {}

/// Represents the message sent to the WorkPlannerAgent.
#[allow(dead_code)]
struct WorkPlannerMessage {}

impl Display for InputMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        let manual_resource_pretty = SchedulerResources(&self.manual_resource);
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
            manual_resource_pretty,
            self.period_lock
        )
    } 
}

#[derive(Deserialize)]
pub struct ManualResource {
    resource: String,
    period: Period,
    capacity: f64
}

#[derive(Deserialize)]
pub struct RawInputMessage {
    message_type: String, 
    name: String,
    platform: String,
    schedule_work_order: Vec<OrderPeriod>,
    unschedule_work_order: HashSet<u32>,
    manual_resource: Vec<ManualResource>,
    period_lock: HashMap<String, bool>
}

impl From<RawInputMessage> for InputMessage {
    fn from(raw: RawInputMessage) -> Self {
        let mut manual_resource_map: HashMap<(String, Period), f64> = HashMap::new();
        for res in raw.manual_resource {
            manual_resource_map.insert((res.resource, res.period), res.capacity);   
        }
    
        InputMessage {
            message_type: raw.message_type,
            name: raw.name,
            platform: raw.platform,
            schedule_work_order: raw.schedule_work_order,
            unschedule_work_order: raw.unschedule_work_order,
            manual_resource: manual_resource_map,
            period_lock: raw.period_lock
        }
    }
}

struct SchedulerResources<'a>(&'a HashMap<(String, Period), f64>);

impl Display for SchedulerResources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        write!(f, "--------------------------")?;
        for ((resource, period), capacity) in self.0 {
            writeln!(f, "Resource: {}, Period: {}, Capacity: {}", resource, period, capacity)?;
        }
        write!(f, "--------------------------")
    }
}