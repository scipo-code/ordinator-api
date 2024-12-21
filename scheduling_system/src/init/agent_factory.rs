use actix::prelude::*;

use anyhow::Result;
use arc_swap::ArcSwap;
use shared_types::operational::OperationalConfiguration;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::strategic::{Periods, StrategicResources};
use shared_types::tactical::{Days, TacticalResources};
use shared_types::Asset;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;
use std::sync::{Arc, MutexGuard};

use crate::agents::operational_agent::algorithm::OperationalAlgorithm;
use crate::agents::operational_agent::{OperationalAgent, OperationalAgentBuilder};
<<<<<<< HEAD
use crate::agents::orchestrator::NotifyOrchestrator;
use crate::agents::strategic_agent::algorithm::strategic_parameters::StrategicParameters;
use crate::agents::strategic_agent::algorithm::PriorityQueues;
use crate::agents::strategic_agent::algorithm::StrategicAlgorithm;
||||||| 83bbf8b
use crate::agents::strategic_agent::strategic_algorithm::OptimizedWorkOrder;
use crate::agents::strategic_agent::strategic_algorithm::PriorityQueues;
use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::strategic_algorithm::{self, OptimizedWorkOrders};
=======
use crate::agents::orchestrator::NotifyOrchestrator;
use crate::agents::strategic_agent::strategic_algorithm::strategic_parameters::StrategicParameters;
use crate::agents::strategic_agent::strategic_algorithm::PriorityQueues;
use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
>>>>>>> d3bf08ac4751bc3e07ab1bec22efb19272c0fba9
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::algorithm::TacticalAlgorithm;
use crate::agents::tactical_agent::TacticalAgent;
use crate::agents::{ArcSwapSharedSolution, SharedSolution};

use shared_types::scheduling_environment::worker_environment::resources::{Id, Resources};
use shared_types::scheduling_environment::SchedulingEnvironment;

#[derive(Debug, Clone)]
pub struct AgentFactory {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl AgentFactory {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        AgentFactory {
            scheduling_environment,
        }
    }

    pub fn create_shared_solution_arc_swap() -> Arc<ArcSwapSharedSolution> {
        let shared_solution_arc_swap = SharedSolution::default();

        Arc::new(ArcSwapSharedSolution(ArcSwap::from(Arc::new(
            shared_solution_arc_swap,
        ))))
    }

    pub fn build_strategic_agent(
        &self,
        asset: Asset,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        shared_solution_arc_swap: Arc<ArcSwapSharedSolution>,
        sender_for_orchestrator: NotifyOrchestrator,
    ) -> Addr<StrategicAgent> {
        let cloned_work_orders = scheduling_environment_guard.work_orders.clone();

        // TODO: We should not clone here! We do not want periods in the strategic agent. I think
        let cloned_periods = scheduling_environment_guard
            .time_environment
            .strategic_periods()
            .clone();

        let period_locks = HashSet::new();

        // period_locks.insert(locked_scheduling_environment.periods()[0].clone());
        // period_locks.insert(locked_scheduling_environment.get_periods()[1].clone());

        let resources_capacity =
            initialize_strategic_resources(scheduling_environment_guard, Work::from(0.0));

        let resources_loading =
            initialize_strategic_resources(scheduling_environment_guard, Work::from(0.0));

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new(), resources_capacity),
            shared_solution_arc_swap,
            period_locks,
            scheduling_environment_guard
                .time_environment
                .strategic_periods()
                .clone(),
        );

        let strategic_resources_from_file = scheduling_environment_guard
            .worker_environment
            .generate_strategic_resources(&cloned_periods);

        strategic_algorithm
            .strategic_parameters
            .strategic_capacity
            .update_resources(strategic_resources_from_file);

        strategic_algorithm.strategic_solution.strategic_loadings = resources_loading;

        strategic_algorithm.create_strategic_parameters(
            &cloned_work_orders,
            &cloned_periods,
            &asset,
        );

        let (sender, receiver) = std::sync::mpsc::channel();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        Arbiter::new().spawn_fn(move || {
            let strategic_addr = StrategicAgent::new(
                asset,
                arc_scheduling_environment,
                strategic_algorithm,
                None,
                sender_for_orchestrator,
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
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        strategic_tactical_optimized_work_orders: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Addr<TacticalAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<TacticalAgent>>();

        let tactical_resources_capacity =
            initialize_tactical_resources(scheduling_environment_guard, Work::from(0.0));

        let tactical_resources_loading =
            initialize_tactical_resources(scheduling_environment_guard, Work::from(0.0));

        let mut tactical_algorithm = TacticalAlgorithm::new(
            scheduling_environment_guard
                .time_environment
                .tactical_days()
                .clone(),
            tactical_resources_capacity,
            tactical_resources_loading,
            strategic_tactical_optimized_work_orders,
        );

        let tactical_resources_from_file = scheduling_environment_guard
            .worker_environment
            .generate_tactical_resources(
                scheduling_environment_guard
                    .time_environment
                    .tactical_days(),
            );

        tactical_algorithm
            .tactical_parameters
            .tactical_capacity
            .update_resources(tactical_resources_from_file);

        tactical_algorithm.create_tactical_parameters(scheduling_environment_guard, &asset);

        let arc_scheduling_environment = self.scheduling_environment.clone();
        Arbiter::new().spawn_fn(move || {
            let tactical_addr = TacticalAgent::new(
                asset,
                0,
                strategic_agent_addr,
                tactical_algorithm,
                arc_scheduling_environment,
                notify_orchestrator,
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
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        number_of_operational_agents: Arc<AtomicU64>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Addr<SupervisorAgent>> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<SupervisorAgent>>();

        let scheduling_environment = Arc::clone(&self.scheduling_environment);

        Arbiter::new().spawn_fn(move || {
            let supervisor_addr = SupervisorAgent::new(
                id_supervisor,
                asset,
                scheduling_environment,
                tactical_agent_addr,
                arc_swap_shared_solution,
                number_of_operational_agents,
                notify_orchestrator,
            )
            .expect("Could not create SupervisorAgent in AgentFactory")
            .start();
            sender.send(supervisor_addr).unwrap();
        });

        Ok(receiver.recv().unwrap())
    }

    pub fn build_operational_agent(
        &self,
        operational_id: Id,
        operational_configuration: &OperationalConfiguration,
        supervisor_agent_addr: HashMap<Id, Addr<SupervisorAgent>>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Addr<OperationalAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<OperationalAgent>>();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        let operational_algorithm =
            OperationalAlgorithm::new(operational_configuration, arc_swap_shared_solution);

        let mut shared_solution_clone = (**operational_algorithm.loaded_shared_solution).clone();

        shared_solution_clone.operational.insert(
            operational_id.clone(),
            operational_algorithm.operational_solution.clone(),
        );

        operational_algorithm
            .arc_swap_shared_solution
            .0
            .store(Arc::new(shared_solution_clone));

        Arbiter::new().spawn_fn(move || {
            let operational_agent_addr = OperationalAgentBuilder::new(
                operational_id,
                arc_scheduling_environment,
                operational_algorithm,
                None,
                supervisor_agent_addr,
                notify_orchestrator,
            )
            .build()
            .start();
            sender.send(operational_agent_addr).unwrap();
        });

        receiver.recv().unwrap()
    }
}

fn initialize_strategic_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: Work,
) -> StrategicResources {
    let mut resource_capacity: HashMap<Resources, Periods> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment
        .get_work_centers()
        .iter()
    {
        let mut periods = HashMap::new();
        for period in scheduling_environment
            .time_environment
            .strategic_periods()
            .iter()
        {
            periods.insert(period.clone(), start_value.clone());
        }
        resource_capacity.insert(resource.clone(), Periods(periods));
    }
    StrategicResources::new(resource_capacity)
}

fn initialize_tactical_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: Work,
) -> TacticalResources {
    let mut resource_capacity: HashMap<Resources, Days> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment
        .get_work_centers()
        .iter()
    {
        let mut days = HashMap::new();
        for day in scheduling_environment
            .time_environment
            .tactical_days()
            .iter()
        {
            days.insert(day.clone(), start_value.clone());
        }
        resource_capacity.insert(resource.clone(), Days::new(days));
    }
    TacticalResources::new(resource_capacity)
}
