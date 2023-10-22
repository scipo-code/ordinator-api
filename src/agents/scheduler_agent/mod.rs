pub mod scheduler_agent;
pub mod scheduler_message;
pub mod scheduler_algorithm;

use crate::models::work_order::WorkOrder;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

use std::collections::HashMap;

pub struct SchedulerAgent {
    platform: String,
    manual_resources : HashMap<(String, Period), f64>,
    backlog: Vec<WorkOrder>,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    periods: Vec<Period>,
}
