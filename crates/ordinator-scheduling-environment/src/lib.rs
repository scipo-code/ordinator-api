pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use serde::{Deserialize, Serialize};
use time_environment::TimeEnvironmentBuilder;
use work_order::{WorkOrders, WorkOrdersBuilder};

use std::fmt;

use self::time_environment::TimeEnvironment;
use self::worker_environment::WorkerEnvironment;

#[derive(Deserialize, Serialize, Debug)]
pub struct SchedulingEnvironment {
    pub work_orders: WorkOrders,
    pub worker_environment: WorkerEnvironment,
    pub time_environment: TimeEnvironment,
    // material
}
pub struct SchedulingEnvironmentBuilder {
    work_orders: Option<WorkOrders>,
    worker_environment: Option<WorkerEnvironment>,
    time_environment: Option<TimeEnvironment>,
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

    // The `SystemConfigurations` needs to be in the `shared_types`.
    // No that is wrong, I think that the `orchestrator` should be in its
    // own module. I think that the whole idea of the `shared_types` is
    // wrong. You should split them out correctly.
    // TODO [ ]
    // git clone a rust project, nushell and see how they structure code.
    // WARN [ ]
    // Good! You took the time to thing through this in a correct way
    // this was a massive payoff. Remember this
    pub fn builder(
        system_configurations: &SystemConfigurations,
        database_connection: D,
    ) -> SchedulingEnvironmentBuilder {
        SchedulingEnvironmentBuilder {
            work_orders: None,
            worker_environment: None,
            time_environment: None,
        }
    }
}

impl SchedulingEnvironmentBuilder {
    pub fn build(self) -> SchedulingEnvironment {
        SchedulingEnvironment {
            work_orders: self
                .work_orders
                .expect("You should build the WorkOrders with the correct parameters injected."),
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
        let mut work_orders_builder = WorkOrders::builder();

        f(&mut work_orders_builder);

        self.work_orders = Some(work_orders_builder.build());
        self
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
