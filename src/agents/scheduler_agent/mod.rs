pub mod scheduler_agent;
pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use actix::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::models::scheduling_environment::WorkOrders;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;

pub struct SchedulerAgent {
    platform: String,
    manual_resources : HashMap<(String, Period), f64>,
    backlog: WorkOrders,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    periods: Vec<Period>,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

impl SchedulerAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }

    // TODO: Here the other Agents Addr messages will also be handled.
}