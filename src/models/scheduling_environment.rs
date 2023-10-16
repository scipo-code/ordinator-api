use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;
use std::collections::HashMap;

use std::fmt;

pub struct WorkOrders {
    pub inner: HashMap<u32, WorkOrder>
}

impl WorkOrders {
    pub fn new() -> Self {
        WorkOrders {
            inner: HashMap::<u32, WorkOrder>::new()
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder) {
        self.inner.insert(work_order.get_work_order_number(), work_order);
    }
}

pub struct SchedulingEnvironment {
    work_orders: WorkOrders,
    worker_environment: WorkerEnvironment,
    // time_and_period
    // material
}

impl SchedulingEnvironment {    
    pub fn new(work_orders: WorkOrders, worker_environment: WorkerEnvironment) -> Self {
        SchedulingEnvironment {
            work_orders,
            worker_environment,
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