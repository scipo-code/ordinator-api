use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::operational_agent::{OperationalAgent, OperationalAgentBuilder};
use crate::agents::strategic_agent::strategic_algorithm::OptimizedWorkOrder;
use crate::agents::strategic_agent::strategic_algorithm::PriorityQueues;
use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::strategic_algorithm::{self, OptimizedWorkOrders};
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::tactical_algorithm::{self, Day, TacticalAlgorithm};
use crate::agents::tactical_agent::TacticalAgent;
use crate::models::time_environment::period::Period;
use crate::models::SchedulingEnvironment;
use crate::models::WorkOrders;
use shared_messages::resources::{Id, Resources};

pub struct AgentFactory {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl AgentFactory {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        AgentFactory {
            scheduling_environment,
        }
    }

    pub fn build_strategic_agent(&self) -> Addr<StrategicAgent> {
        let mut cloned_work_orders = self
            .scheduling_environment
            .lock()
            .unwrap()
            .clone_work_orders();
        let cloned_periods = self
            .scheduling_environment
            .lock()
            .unwrap()
            .clone_strategic_periods();
        let optimized_work_orders: OptimizedWorkOrders =
            create_optimized_work_orders(&mut cloned_work_orders, &cloned_periods);

        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let mut period_locks = HashSet::new();

        period_locks.insert(locked_scheduling_environment.periods()[0].clone());
        //period_locks.insert(locked_scheduling_environment.get_periods()[1].clone());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            initialize_strategic_resources(&locked_scheduling_environment, 0.0),
            initialize_strategic_resources(&locked_scheduling_environment, 0.0),
            PriorityQueues::new(),
            optimized_work_orders,
            period_locks,
            locked_scheduling_environment.clone_strategic_periods(),
        );

        drop(locked_scheduling_environment);

        scheduler_agent_algorithm.calculate_objective();

        let (sender, receiver) = std::sync::mpsc::channel();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        Arbiter::new().spawn_fn(move || {
            let strategic_addr = StrategicAgent::new(
                String::from("Dan F"),
                arc_scheduling_environment,
                scheduler_agent_algorithm,
                None,
            )
            .start();
            sender.send(strategic_addr).unwrap();
        });
        dbg!();
        receiver.recv().unwrap()
    }

    pub fn build_tactical_agent(
        &self,
        time_horizon: u32,
        strategic_agent_addr: Addr<StrategicAgent>,
    ) -> Addr<TacticalAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<TacticalAgent>>();
        dbg!();
        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
        dbg!();
        let tactical_algorithm = TacticalAlgorithm::new(
            scheduling_environment_guard.clone_tactical_days(),
            initialize_tactical_resources(&scheduling_environment_guard, 0.0),
            initialize_tactical_resources(&scheduling_environment_guard, 0.0),
        );

        drop(scheduling_environment_guard);
        dbg!();
        let arc_scheduling_environment = self.scheduling_environment.clone();
        Arbiter::new().spawn_fn(move || {
            dbg!("Tactical agent created");
            let tactical_addr = TacticalAgent::new(
                0,
                time_horizon,
                strategic_agent_addr,
                tactical_algorithm,
                arc_scheduling_environment,
            )
            .start();
            sender.send(tactical_addr).unwrap();
        });
        receiver.recv().unwrap()
    }

    pub fn build_supervisor_agent(
        &self,
        id: Id,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> Addr<SupervisorAgent> {
        let supervisor_agent =
            SupervisorAgent::new(id, tactical_agent_addr, self.scheduling_environment.clone());
        supervisor_agent.start()
    }

    pub fn build_operational_agent(
        &self,
        id: Id,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Addr<OperationalAgent> {
        let operational_agent = OperationalAgentBuilder::new(
            id,
            self.scheduling_environment.clone(),
            supervisor_agent_addr,
        )
        .build();
        operational_agent.start()
    }
}

/// This function should be used by the scheduling environment. It should not be used by the
/// algorithm itself.
fn create_optimized_work_orders(
    work_orders: &mut WorkOrders,
    periods: &[Period],
) -> OptimizedWorkOrders {
    let mut optimized_work_orders: HashMap<u32, OptimizedWorkOrder> = HashMap::new();

    let last_period = periods.last();
    for (work_order_number, work_order) in &mut work_orders.inner {
        let mut excluded_periods: HashSet<Period> = HashSet::new();

        for (i, period) in periods.iter().enumerate() {
            if period < &work_order.order_dates_mut().earliest_allowed_start_period {
                excluded_periods.insert(period.clone());
            } else if work_order.is_vendor() && i <= 3 {
                excluded_periods.insert(period.clone());
            } else if work_order.revision().shutdown && i <= 3 {
                excluded_periods.insert(period.clone());
            }
        }

        if work_order.is_vendor() {
            optimized_work_orders.insert(
                *work_order_number,
                OptimizedWorkOrder::new(
                    periods.last().cloned(),
                    periods.last().cloned(),
                    excluded_periods.clone(),
                    None,
                    work_order.work_order_weight(),
                    work_order.work_load().clone(),
                ),
            );
        }

        if work_order.unloading_point().present {
            let period = work_order.unloading_point().period.clone();
            optimized_work_orders.insert(
                *work_order_number,
                OptimizedWorkOrder::new(
                    period.clone(),
                    period,
                    excluded_periods.clone(),
                    None,
                    work_order.work_order_weight(),
                    work_order.work_load().clone(),
                ),
            );
            continue;
        }
        optimized_work_orders.insert(
            *work_order_number,
            OptimizedWorkOrder::new(
                last_period.cloned(),
                None,
                excluded_periods,
                Some(
                    work_order
                        .order_dates_mut()
                        .latest_allowed_finish_period
                        .clone(),
                ),
                work_order.work_order_weight(),
                work_order.work_load().clone(),
            ),
        );
    }
    OptimizedWorkOrders::new(optimized_work_orders)
}

fn initialize_strategic_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: f64,
) -> strategic_algorithm::AlgorithmResources {
    let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment()
        .get_work_centers()
        .iter()
    {
        let mut periods = HashMap::new();
        for period in scheduling_environment.periods().iter() {
            periods.insert(period.clone(), start_value);
        }
        resource_capacity.insert(resource.clone(), periods);
    }
    strategic_algorithm::AlgorithmResources::new(resource_capacity)
}

fn initialize_tactical_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: f64,
) -> tactical_algorithm::AlgorithmResources {
    let mut resource_capacity: HashMap<Resources, HashMap<Day, f64>> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment()
        .get_work_centers()
        .iter()
    {
        let mut days = HashMap::new();
        for day in scheduling_environment.clone_tactical_days().iter() {
            days.insert(day.clone(), start_value);
        }
        resource_capacity.insert(resource.clone(), days);
    }
    dbg!(resource_capacity.clone());
    tactical_algorithm::AlgorithmResources::new(resource_capacity)
}
