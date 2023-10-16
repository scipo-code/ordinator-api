use std::collections::HashMap;
use crate::agents::work_center_agent::WorkCenterAgent;
use crate::models::work_order::WorkOrder;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

pub struct SchedulerAgent {
    platform: String,
    workcenter_agents: HashMap<String, WorkCenterAgent>,
    backlog: Vec<WorkOrder>,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    // inbox: Receiver<SchedulerMessage>,  // Using an mpsc channel for message passing
    periods: Vec<Period>,
}