pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use time_environment::TimeEnvironmentBuilder;
use work_order::WorkOrderBuilder;

use crate::scheduling_environment::time_environment::period::Period;
use crate::scheduling_environment::work_order::WorkOrder;
use crate::scheduling_environment::worker_environment::WorkerEnvironment;
use crate::Asset;

use self::time_environment::TimeEnvironment;
use self::work_order::operation::Operation;
use self::work_order::{WorkOrderActivity, WorkOrderNumber};

#[derive(Deserialize, Serialize, Debug)]
pub struct SchedulingEnvironment {
    pub work_orders: WorkOrders,
    pub worker_environment: WorkerEnvironment,
    pub time_environment: TimeEnvironment,
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

    pub fn operation(&self, work_order_activity: &WorkOrderActivity) -> &Operation {
        self.work_orders.inner.get(&work_order_activity.0).expect("WorkOrder not found in SchedulinEnvironment. User is responsible for calling this method with the right arguments").operations.get(&work_order_activity.1).expect("ActivityNumber is not present in the WorkOrder")
    }

    pub fn initialize_work_orders(&mut self, periods: &[Period]) {
        for (_, work_order) in self.work_orders.inner.iter_mut() {
            work_order.initialize(periods);
        }
    }

    pub fn builder() -> SchedulingEnvironmentBuilder {
        SchedulingEnvironmentBuilder::new()
    }
}

#[derive(Default)]
pub struct SchedulingEnvironmentBuilder {
    pub work_orders: Option<WorkOrders>,
    pub worker_environment: Option<WorkerEnvironment>,
    pub time_environment: Option<TimeEnvironment>,
}

impl SchedulingEnvironmentBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> SchedulingEnvironment {
        SchedulingEnvironment {
            work_orders: self.work_orders.unwrap_or_default(),
            worker_environment: self.worker_environment.unwrap_or_default(),
            time_environment: self.time_environment.unwrap_or_default(),
        }
    }

    pub fn time_environment(&mut self, time_environment: TimeEnvironment) -> &mut Self {
        self.time_environment = Some(time_environment);
        self
    }

    pub fn time_environment_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut TimeEnvironmentBuilder) -> &mut TimeEnvironmentBuilder,
    {
        let mut time_environment_builder = TimeEnvironmentBuilder::default();

        f(&mut time_environment_builder);

        self.time_environment = Some(time_environment_builder.build());
        self
    }

    pub fn worker_environment(&mut self, worker_environment: WorkerEnvironment) -> &mut Self {
        self.worker_environment = Some(worker_environment);
        self
    }
    pub fn work_orders(&mut self, work_orders: WorkOrders) -> &mut Self {
        self.work_orders = Some(work_orders);
        self
    }

    pub fn work_orders_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrdersBuilder) -> &mut WorkOrdersBuilder,
    {
        let mut work_orders_builder = WorkOrdersBuilder::default();

        f(&mut work_orders_builder);

        self.work_orders = Some(work_orders_builder.build());
        self
    }
}

impl Default for SchedulingEnvironment {
    fn default() -> Self {
        SchedulingEnvironment {
            work_orders: WorkOrders::default(),
            worker_environment: WorkerEnvironment::new(),
            time_environment: TimeEnvironment::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WorkOrders {
    pub inner: HashMap<WorkOrderNumber, WorkOrder>,
}

#[derive(Default)]
pub struct WorkOrdersBuilder {
    inner: Option<HashMap<WorkOrderNumber, WorkOrder>>,
}

impl WorkOrdersBuilder {
    pub fn build(self) -> WorkOrders {
        WorkOrders {
            inner: self.inner.unwrap_or_default(),
        }
    }

    pub fn work_order_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderBuilder) -> &mut WorkOrderBuilder,
    {
        let mut work_order_builder = WorkOrderBuilder::default();

        f(&mut work_order_builder);

        work_order_builder.inner = Some(work_order_builder.build());
        self
    }
}

impl WorkOrders {
    pub fn insert(&mut self, work_order: WorkOrder) {
        self.inner.insert(work_order.work_order_number, work_order);
    }

    pub fn new_work_order(&self, work_order_number: WorkOrderNumber) -> bool {
        !self.inner.contains_key(&work_order_number)
    }

    pub fn work_orders_by_asset(&self, asset: &Asset) -> HashMap<&WorkOrderNumber, &WorkOrder> {
        self.inner
            .iter()
            .filter(|(_, wo)| &wo.work_order_info.functional_location.asset == asset)
            .collect()
    }
}

impl FromIterator<(WorkOrderNumber, WorkOrder)> for WorkOrders {
    fn from_iter<T: IntoIterator<Item = (WorkOrderNumber, WorkOrder)>>(iter: T) -> Self {
        let mut work_orders = HashMap::new();

        for (work_order_number, work_order) in iter {
            work_orders.insert(work_order_number, work_order);
        }
        WorkOrders { inner: work_orders }
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let workers = self.worker_environment.agent_environment.operational.len();
        write!(
            f,
            "The Scheduling Environment is currently comprised of
        \n  number of work orders: {}
        \n  number of worker entries: {}
        \n  number of strategic periods: {}, 
        \n  number of tactical days: {}",
            self.work_orders.inner.len(),
            workers,
            self.time_environment.strategic_periods.len(),
            self.time_environment.tactical_days.len(),
        )?;
        Ok(())
    }
}
