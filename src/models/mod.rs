pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use actix::prelude::*;
use std::collections::HashMap;
use std::fmt;

use crate::api::websocket_agent::{TimeEnvironmentMessage, WebSocketAgent};
use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;

pub struct SchedulingEnvironment {
    pub work_orders: WorkOrders,
    worker_environment: WorkerEnvironment,
    pub periods: Vec<Period>,
    web_socket_agent_addr_option: Option<Addr<WebSocketAgent>>,
    // material
}

impl SchedulingEnvironment {
    pub fn new(
        work_orders: WorkOrders,
        worker_environment: WorkerEnvironment,
        periods: Vec<Period>,
        web_socket_agent_addr: Option<Addr<WebSocketAgent>>,
    ) -> Self {
        SchedulingEnvironment {
            work_orders,
            worker_environment,
            periods,
            web_socket_agent_addr_option: web_socket_agent_addr,
        }
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.periods = periods;
        let message = TimeEnvironmentMessage {
            frontend_message_type: String::from("time_environment"),
            time: chrono::Utc::now(),
            periods: self.periods.clone(),
        };
        match &self.web_socket_agent_addr_option {
            Some(ws_addr) => ws_addr.do_send(message),
            None => panic!("SchedulingEnvironment does not have a WebSocketAgent Address"),
        }
    }
}

impl Default for SchedulingEnvironment {
    fn default() -> Self {
        SchedulingEnvironment {
            work_orders: WorkOrders::new(),
            worker_environment: WorkerEnvironment::new(),
            periods: vec![Period::new(
                0,
                chrono::Utc::now(),
                chrono::Utc::now() + chrono::Duration::days(14) - chrono::Duration::seconds(1),
            )],
            web_socket_agent_addr_option: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkOrders {
    pub inner: HashMap<u32, WorkOrder>,
}

impl WorkOrders {
    pub fn new() -> Self {
        WorkOrders {
            inner: HashMap::<u32, WorkOrder>::new(),
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder) {
        self.inner
            .insert(work_order.get_work_order_number(), work_order);
    }
}

impl WorkOrders {
    pub fn initialize_work_orders(&mut self) {
        for (_, work_order) in self.inner.iter_mut() {
            work_order.initialize();
        }
    }
}

impl WorkOrders {
    pub fn new_work_order(&self, order_number: u32) -> bool {
        !self.inner.contains_key(&order_number)
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The Scheduling Environment is currently comprised of \n  work_orders: {},\n  number of worker entries: {}", self.work_orders.inner.len(), self.worker_environment.crew.workers.len())
    }
}

impl fmt::Display for WorkOrders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The Work Orders are currently comprised of \n  work_orders: {}",
            self.inner.len()
        )?;
        for (_, work_order) in self.inner.iter() {
            write!(f, "{}", work_order)?;
        }
        Ok(())
    }
}
