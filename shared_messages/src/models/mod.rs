pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::collections::HashMap;
use std::fmt;

use crate::models::time_environment::day::Day;
use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;

use self::time_environment::TimeEnvironment;

pub struct SchedulingEnvironment {
    work_orders: WorkOrders,
    worker_environment: WorkerEnvironment,
    time_environment: TimeEnvironment,
    // material
}

impl SchedulingEnvironment {
    pub fn new(
        work_orders: WorkOrders,
        worker_environment: WorkerEnvironment,
        time_environment: TimeEnvironment,
    ) -> Self {
        SchedulingEnvironment {
            work_orders,
            worker_environment,
            time_environment,
        }
    }

    pub fn clone_strategic_periods(&self) -> Vec<Period> {
        self.time_environment.strategic_periods().clone()
    }

    pub fn tactical_days(&self) -> &Vec<Day> {
        self.time_environment.tactical_days()
    }

    pub fn tactical_periods(&self) -> &Vec<Period> {
        &self.time_environment.tactical_periods
    }

    pub fn clone_work_orders(&self) -> WorkOrders {
        self.work_orders.clone()
    }

    pub fn initialize_work_orders(&mut self, periods: &[Period]) {
        for (_, work_order) in self.work_orders.inner.iter_mut() {
            work_order.initialize(periods);
        }
    }

    pub fn periods_mut(&mut self) -> &mut Vec<Period> {
        &mut self.time_environment.strategic_periods
    }

    pub fn periods(&self) -> &Vec<Period> {
        self.time_environment.strategic_periods()
    }

    pub fn worker_environment(&self) -> &WorkerEnvironment {
        &self.worker_environment
    }

    pub fn initialize_worker_environment(&mut self) {
        self.worker_environment.initialize();
    }

    pub fn work_orders(&self) -> &WorkOrders {
        &self.work_orders
    }

    pub fn work_orders_mut(&mut self) -> &mut WorkOrders {
        &mut self.work_orders
    }

    pub fn time_environment(&self) -> &TimeEnvironment {
        &self.time_environment
    }
}

impl Default for SchedulingEnvironment {
    fn default() -> Self {
        SchedulingEnvironment {
            work_orders: WorkOrders::new(),
            worker_environment: WorkerEnvironment::new(),
            time_environment: TimeEnvironment::new(Vec::new(), Vec::new(), Vec::new()),
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
            .insert(*work_order.work_order_number(), work_order);
    }

    pub fn new_work_order(&self, order_number: u32) -> bool {
        !self.inner.contains_key(&order_number)
    }
}

impl FromIterator<(u32, WorkOrder)> for WorkOrders {
    fn from_iter<T: IntoIterator<Item = (u32, WorkOrder)>>(iter: T) -> Self {
        let mut work_orders = HashMap::new();

        for (work_order_number, work_order) in iter {
            work_orders.insert(work_order_number, work_order);
        }
        WorkOrders { inner: work_orders }
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let workers = match self.worker_environment().get_crew().as_ref() {
            Some(crew) => crew.get_workers().len(),
            None => 0,
        };

        write!(
            f,
            "The Scheduling Environment is currently comprised of
        \n  number of work orders: {}
        \n  number of worker entries: {}
        \n  number of strategic periods: {}, 
        \n  number of tactical days: {}",
            self.work_orders.inner.len(),
            workers,
            self.time_environment().strategic_periods().len(),
            self.time_environment().tactical_days().len(),
        )?;
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
        for (i, work_order) in self.inner.values().enumerate() {
            if i % 10 == 0 {
                writeln!(
                f,
                "                          |EARL-PERIOD| SCH|AWSC|SECE|REVISION|TYPE|PRIO|VEN*| MAT|  Unloading|Asset|",
            )
            .unwrap();
            };
            write!(f, "{}", work_order.to_string_normal())?;
        }
        Ok(())
    }
}
