use actix::prelude::*;

use arc_swap::ArcSwap;
use shared_types::operational::OperationalConfiguration;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::strategic::{strategic_request_scheduling_message, Periods, StrategicResources};
use shared_types::tactical::{Days, TacticalResources};
use shared_types::Asset;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::operational_agent::algorithm::{OperationalAlgorithm, OperationalObjective};
use crate::agents::operational_agent::{OperationalAgent, OperationalAgentBuilder};
use crate::agents::strategic_agent::strategic_algorithm::optimized_work_orders::StrategicParameters;
use crate::agents::strategic_agent::strategic_algorithm::PriorityQueues;
use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::tactical_algorithm::{
    TacticalAlgorithm, TacticalOperation, TacticalParameters,
};
use crate::agents::tactical_agent::TacticalAgent;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::agents::StrategicTacticalSolutionArcSwap;

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

    pub fn create_arc_swap_for_strategic_tactical() -> Arc<StrategicTacticalSolutionArcSwap> {
        let strategic_tactical_solution_arc_swap = SharedSolution {
            strategic: MetaStrategic::default(),
            tactical: MetaTactical::default(),
        };

        Arc::new(StrategicTacticalSolutionArcSwap(ArcSwap::from(Arc::new(
            strategic_tactical_solution_arc_swap,
        ))))
    }

    pub fn build_strategic_agent(
        &self,
        asset: Asset,
        strategic_resources: Option<StrategicResources>,
        strategic_tactical_optimized_work_orders: Arc<StrategicTacticalSolutionArcSwap>,
    ) -> Addr<StrategicAgent> {
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

        let mut resources_capacity =
            initialize_strategic_resources(&locked_scheduling_environment, Work::from(0.0));

        if let Some(resources) = strategic_resources {
            resources_capacity.update_resources(resources);
        }

        let resources_loading =
            initialize_strategic_resources(&locked_scheduling_environment, Work::from(0.0));

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            resources_capacity,
            resources_loading,
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new()),
            strategic_tactical_optimized_work_orders,
            period_locks,
            locked_scheduling_environment.clone_strategic_periods(),
        );

        strategic_agent_algorithm.create_strategic_parameters(
            &mut cloned_work_orders,
            &cloned_periods,
            &asset,
        );

        drop(locked_scheduling_environment);

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
        tactical_resources: Option<TacticalResources>,
        strategic_tactical_optimized_work_orders: Arc<StrategicTacticalSolutionArcSwap>,
    ) -> Addr<TacticalAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<TacticalAgent>>();

        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

        let tactical_periods = scheduling_environment_guard.tactical_periods().clone();

        let mut tactical_resources_capacity =
            initialize_tactical_resources(&scheduling_environment_guard, Work::from(0.0));

        if let Some(resources) = tactical_resources {
            tactical_resources_capacity.update_resources(resources);
        }

        let tactical_resources_loading =
            initialize_tactical_resources(&scheduling_environment_guard, Work::from(0.0));

        let mut tactical_algorithm = TacticalAlgorithm::new(
            scheduling_environment_guard.tactical_days().clone(),
            tactical_periods.clone(),
            tactical_resources_capacity,
            tactical_resources_loading,
            strategic_tactical_optimized_work_orders,
        );

        tactical_algorithm.create_tactical_parameters(&scheduling_environment_guard, &asset);

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
        number_of_operational_agents: Arc<AtomicU64>,
    ) -> Addr<SupervisorAgent> {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<SupervisorAgent>>();

        let scheduling_environment = Arc::clone(&self.scheduling_environment);

        Arbiter::new().spawn_fn(move || {
            let supervisor_addr = SupervisorAgent::new(
                id_supervisor,
                asset,
                scheduling_environment,
                tactical_agent_addr,
                number_of_operational_agents,
            )
            .start();
            sender.send(supervisor_addr).unwrap();
        });

        receiver.recv().unwrap()
    }

    pub fn build_operational_agent(
        &self,
        id_operational: Id,
        operational_configuration: OperationalConfiguration,
        supervisor_agent_addr: HashMap<Id, Addr<SupervisorAgent>>,
    ) -> (OperationalObjective, Addr<OperationalAgent>) {
        let (sender, receiver) = std::sync::mpsc::channel::<Addr<OperationalAgent>>();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        let operational_algorithm = OperationalAlgorithm::new(operational_configuration.clone());

        let operational_objective = Arc::clone(&operational_algorithm.objective_value);
        Arbiter::new().spawn_fn(move || {
            let operational_agent_addr = OperationalAgentBuilder::new(
                id_operational,
                arc_scheduling_environment,
                operational_configuration,
                operational_algorithm,
                None,
                supervisor_agent_addr,
            )
            .build()
            .start();
            sender.send(operational_agent_addr).unwrap();
        });

        (operational_objective, receiver.recv().unwrap())
    }
}

fn initialize_strategic_resources(
    scheduling_environment: &SchedulingEnvironment,
    start_value: Work,
) -> StrategicResources {
    let mut resource_capacity: HashMap<Resources, Periods> = HashMap::new();
    for resource in scheduling_environment
        .worker_environment()
        .get_work_centers()
        .iter()
    {
        let mut periods = HashMap::new();
        for period in scheduling_environment.periods().iter() {
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
        .worker_environment()
        .get_work_centers()
        .iter()
    {
        let mut days = HashMap::new();
        for day in scheduling_environment.tactical_days().iter() {
            days.insert(day.clone(), start_value.clone());
        }
        resource_capacity.insert(resource.clone(), Days::new(days));
    }
    TacticalResources::new(resource_capacity)
}
