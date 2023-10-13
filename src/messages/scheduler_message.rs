use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::models::order_period::OrderPeriod;
use std::fmt::{self, Display};

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
        write!(f, 
            "Name: {}, 
            \nPlatform: {}, 
            \nSchedule Work Order: {}, 
            \nUnschedule Work Order: {:?}, 
            \nPeriod Lock: {:?}", 
            self.name, 
            self.platform, 
            self.schedule_work_order.len(), 
            self.unschedule_work_order, 
            self.period_lock
        )
    } 
}
