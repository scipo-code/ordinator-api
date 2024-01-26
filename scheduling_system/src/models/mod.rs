pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use actix::prelude::*;
use std::collections::HashMap;
use std::fmt;
use tracing::info;

use crate::agents::scheduler_agent::scheduler_message::PeriodMessage;
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;

pub struct SchedulingEnvironment {
    work_orders: WorkOrders,
    worker_environment: WorkerEnvironment,
    periods: Vec<Period>,
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
        let message = PeriodMessage {
            frontend_message_type: String::from("frontend_scheduler_periods"),
            periods: self.periods.clone(),
        };
        match &self.web_socket_agent_addr_option {
            Some(ws_addr) => ws_addr.do_send(message),
            None => info!("No WebSocketAgent address has been provided yet."),
        }
    }
    
    pub fn clone_periods(&self) -> Vec<Period> {
        self.periods.clone()
    }

    pub fn clone_work_orders(&self) -> WorkOrders {
        self.work_orders.clone()
    }

    pub fn initialize_work_orders(&mut self) {
        for (_, work_order) in self.work_orders.inner.iter_mut() {
            work_order.initialize();
        }
    }

    pub fn get_mut_periods(&mut self) -> &mut Vec<Period> {
        &mut self.periods
    }

    pub fn get_periods(&self) -> &Vec<Period> {
        &self.periods
    }

    pub fn get_worker_environment(&self) -> &WorkerEnvironment {
        &self.worker_environment
    }

}

impl SchedulingEnvironment {
    pub fn initialize_worker_environment(&mut self) {
        self.worker_environment.initialize();
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
    pub fn new_work_order(&self, order_number: u32) -> bool {
        !self.inner.contains_key(&order_number)
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {                 
        write!(f, "The Scheduling Environment is currently comprised of \n  number of work orders: {},\n  number of worker entries: {},\n  number of periods: {}", 
        self.work_orders.inner.len(), 
        match self.get_worker_environment().get_crew().as_ref() {
            Some(crew) => crew.get_workers().len(),
            None => 0,
    }, self.periods.len())?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    impl SchedulingEnvironment {
        pub fn get_work_orders(&self) -> &WorkOrders {
            &self.work_orders
        }
    }
}
