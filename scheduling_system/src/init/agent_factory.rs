use actix::prelude::*;
use shared_messages::models::time_environment::day::Day;
use shared_messages::strategic::StrategicResources;
use shared_messages::tactical::{Days, TacticalResources};
use shared_messages::Asset;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::operational_agent::{OperationalAgent, OperationalAgentBuilder};
use crate::agents::strategic_agent::strategic_algorithm::optimized_work_orders::OptimizedWorkOrders;
use crate::agents::strategic_agent::strategic_algorithm::PriorityQueues;
use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::tactical_algorithm::{self, TacticalAlgorithm};
use crate::agents::tactical_agent::TacticalAgent;
use crate::agents::traits::LargeNeighborHoodSearch;
use shared_messages::models::time_environment::period::Period;
use shared_messages::models::worker_environment::resources::{Id, Resources};
use shared_messages::models::SchedulingEnvironment;

pub struct AgentFactory {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl AgentFactory {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        AgentFactory {
            scheduling_environment,
        }
    }

    pub fn build_strategic_agent(&self, asset: Asset) -> Addr<StrategicAgent> {
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

        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let period_locks = HashSet::new();

        // period_locks.insert(locked_scheduling_environment.periods()[0].clone());
        // period_locks.insert(locked_scheduling_environment.get_periods()[1].clone());

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            initialize_strategic_resources(&locked_scheduling_environment, 0.0),
            initialize_strategic_resources(&locked_scheduling_environment, 0.0),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            period_locks,
            locked_scheduling_environment.clone_strategic_periods(),
        );

        strategic_agent_algorithm.create_optimized_work_orders(
            &mut cloned_work_orders,
            &cloned_periods,
            &asset,
        );
        drop(locked_scheduling_environment);

        strategic_agent_algorithm.calculate_objective_value();

        let (sender, receiver) = std::sync::mpsc::channel();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        Arbiter::new().spawn_fn(move || {
            let strategic_addr = StrategicAgent::new(
                asset,
                arc_scheduling_environment,
                strategic_agent_algorithm,
                None,
            )
            .start();
            sender.send(strategic_addr).unwrap();
        });

        receiver.recv().unwrap()
    }

    pub fn build_tactical_agent(
        &self,
        asset: Asset,
        strategic_agent_addr: Addr<StrategicAgent>,
    ) -> Addr<TacticalAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<TacticalAgent>>();

        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

        let tactical_periods = scheduling_environment_guard.tactical_periods().clone();

        let tactical_algorithm = TacticalAlgorithm::new(
            scheduling_environment_guard.tactical_days().clone(),
            tactical_periods.clone(),
            initialize_tactical_resources(&scheduling_environment_guard, 0.0),
            initialize_tactical_resources(&scheduling_environment_guard, 0.0),
        );

        drop(scheduling_environment_guard);

        let arc_scheduling_environment = self.scheduling_environment.clone();
        Arbiter::new().spawn_fn(move || {
            let tactical_addr = TacticalAgent::new(
                asset,
                0,
                tactical_periods,
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
        asset: Asset,
        id_supervisor: Id,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> Addr<SupervisorAgent> {
        let supervisor_agent = SupervisorAgent::new(
            id_supervisor,
            asset,
            self.scheduling_environment.clone(),
            tactical_agent_addr,
        );
        supervisor_agent.start()
    }

    pub fn build_operational_agent(
        &self,
        id_operational: Id,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Addr<OperationalAgent> {
        let operational_agent = OperationalAgentBuilder::new(
            id_operational,
            self.scheduling_environment.clone(),
            supervisor_agent_addr,
        )
        .build();
        operational_agent.start()
    }
}

fn initialize_strategic_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: f64,
) -> StrategicResources {
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
    StrategicResources::new(resource_capacity)
}

fn initialize_tactical_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: f64,
) -> TacticalResources {
    let mut resource_capacity: HashMap<Resources, Days> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment()
        .get_work_centers()
        .iter()
    {
        let mut days = HashMap::new();
        for day in scheduling_environment.tactical_days().iter() {
            days.insert(day.clone(), start_value);
        }
        resource_capacity.insert(resource.clone(), Days::new(days));
    }
    TacticalResources::new(resource_capacity)
}
