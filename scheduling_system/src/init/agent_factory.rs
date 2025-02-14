use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use priority_queue::PriorityQueue;
use shared_types::operational::{
    OperationalConfiguration, OperationalRequestMessage, OperationalResponseMessage,
};
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::strategic::{
    StrategicRequestMessage, StrategicResources, StrategicResponseMessage,
};
use shared_types::supervisor::{SupervisorRequestMessage, SupervisorResponseMessage};
use shared_types::tactical::{
    Days, TacticalRequestMessage, TacticalResources, TacticalResponseMessage,
};
use shared_types::Asset;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::{Arc, MutexGuard};

use crate::agents::operational_agent::algorithm::operational_parameter::OperationalParameter;
use crate::agents::operational_agent::algorithm::OperationalAlgorithm;
use crate::agents::operational_agent::OperationalOptions;
use crate::agents::orchestrator::{Communication, NotifyOrchestrator};
use crate::agents::strategic_agent::algorithm::strategic_parameters::{
    StrategicClustering, StrategicParameters,
};
use crate::agents::strategic_agent::algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::StrategicOptions;
use crate::agents::supervisor_agent::algorithm::delegate::Delegate;
use crate::agents::supervisor_agent::algorithm::SupervisorAlgorithm;
use crate::agents::supervisor_agent::SupervisorOptions;
use crate::agents::tactical_agent::algorithm::TacticalAlgorithm;
use crate::agents::tactical_agent::TacticalOptions;
use crate::agents::{Agent, AgentMessage, ArcSwapSharedSolution, SharedSolution};

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
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<AgentMessage<StrategicRequestMessage>, StrategicResponseMessage>>
    {
        let cloned_work_orders = scheduling_environment_guard.work_orders.clone();

        // TODO: We should not clone here! We do not want periods in the strategic agent. I think
        let cloned_periods = scheduling_environment_guard
            .time_environment
            .strategic_periods()
            .clone();

        let period_locks = HashSet::new();

        // period_locks.insert(locked_scheduling_environment.periods()[0].clone());
        // period_locks.insert(locked_scheduling_environment.get_periods()[1].clone());

        let mut strategic_clustering = StrategicClustering::default();
        strategic_clustering.calculate_clustering_values(&asset, &cloned_work_orders)?;

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueue::new(),
            StrategicParameters::new(
                HashMap::new(),
                StrategicResources::default(),
                strategic_clustering,
            ),
            shared_solution_arc_swap,
            period_locks,
            scheduling_environment_guard
                .time_environment
                .strategic_periods()
                .clone(),
        );

        let strategic_resources_from_work_environment = scheduling_environment_guard
            .worker_environment
            .generate_strategic_resources(&cloned_periods);

        strategic_algorithm
            .strategic_parameters
            .strategic_capacity
            .update_resource_capacities(strategic_resources_from_work_environment.clone())
            .with_context(|| {
                format!(
                    "Could not initialize the initialial StrategicResources. Line: {}",
                    line!()
                )
            })?;

        // These loadings should come from the SchedulingEnvironment
        strategic_algorithm
            .strategic_solution
            .strategic_loadings
            .initialize_resource_loadings(strategic_resources_from_work_environment.clone());

        strategic_algorithm.create_strategic_parameters(
            &cloned_work_orders,
            &cloned_periods,
            &asset,
        );

        for work_order_number in strategic_algorithm
            .strategic_parameters
            .strategic_work_order_parameters
            .keys()
        {
            strategic_algorithm
                .strategic_solution
                .strategic_scheduled_work_orders
                .insert(*work_order_number, None);
        }

        let arc_scheduling_environment = self.scheduling_environment.clone();

        // FIX
        // This whole build process should be encapsulated! I think that this is the best way of doing things.
        let strategic_id = Id::new("StrategicAgent".to_string(), vec![], None);

        // This send
        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();

        let mut strategic_agent = Agent::new(
            asset.clone(),
            strategic_id,
            arc_scheduling_environment,
            strategic_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        let options = StrategicOptions::default();

        // FIX
        // Turn this into a std::thread::spawn and work on that to make the program function correctly.
        std::thread::Builder::new()
            .name(format!(
                "{} for Asset: {}",
                std::any::type_name::<StrategicAlgorithm>(),
                asset,
            ))
            .spawn(move || strategic_agent.run(options))?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }

    pub fn build_tactical_agent(
        &self,
        asset: &Asset,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        strategic_tactical_optimized_work_orders: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<AgentMessage<TacticalRequestMessage>, TacticalResponseMessage>> {
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

        tactical_algorithm.create_tactical_parameters(scheduling_environment_guard, asset);

        let id = Id("Tactical".to_string(), vec![], None);

        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        let mut tactical_agent = Agent::new(
            asset.clone(),
            id,
            arc_scheduling_environment,
            tactical_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        let options = TacticalOptions::default();

        std::thread::Builder::new()
            .name(format!(
                "{} for Asset: {}",
                std::any::type_name::<TacticalAlgorithm>(),
                asset,
            ))
            .spawn(move || tactical_agent.run(options))?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }

    pub fn build_supervisor_agent(
        &self,
        asset: &Asset,
        id_supervisor: &Id,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<AgentMessage<SupervisorRequestMessage>, SupervisorResponseMessage>>
    {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();
        let supervisor_periods = scheduling_environment
            .time_environment
            .supervisor_periods
            .as_slice();

        let mut supervisor_algorithm = SupervisorAlgorithm::new(
            id_supervisor.clone().1,
            arc_swap_shared_solution,
            supervisor_periods,
        );

        for (work_order_number, work_order) in &scheduling_environment.work_orders.inner {
            for (activity_number, operation) in &work_order.operations {
                let work_order_activity = &(*work_order_number, *activity_number);
                supervisor_algorithm
                    .supervisor_parameters
                    .create_and_insert_supervisor_parameter(operation, work_order_activity);

                for operational_agent in supervisor_algorithm
                    .loaded_shared_solution
                    .operational
                    .keys()
                {
                    if operational_agent.1.contains(
                        &supervisor_algorithm
                            .supervisor_parameters
                            .supervisor_parameter(work_order_activity)
                            .context("The SupervisorParameter was not found")?
                            .resource,
                    ) {
                        let operation = scheduling_environment.operation(work_order_activity);
                        let delegate = Delegate::build(operation);
                        supervisor_algorithm
                            .supervisor_solution
                            .insert_supervisor_solution(
                                operational_agent,
                                delegate,
                                *work_order_activity,
                            )
                            .context(
                                "Supervisor could not insert operational solution correctly",
                            )?;
                    }
                }
            }
        }

        let scheduling_environment = Arc::clone(&self.scheduling_environment);
        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();
        // It is the `Algorithm` that should have the arc_swap_shared_solution, not the
        // `Agent`. The mutable requirements makes a lot of sense.
        let mut supervisor_agent = Agent::new(
            asset.clone(),
            id_supervisor.clone(),
            scheduling_environment,
            supervisor_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        let options = SupervisorOptions::default();

        std::thread::Builder::new()
            .name(format!(
                "{} for Asset: {} for Id: {}",
                std::any::type_name::<SupervisorAlgorithm>(),
                asset,
                &id_supervisor,
            ))
            .spawn(move || supervisor_agent.run(options))?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }

    pub fn build_operational_agent(
        &self,
        asset: &Asset,
        operational_id: &Id,
        // FIX
        // This should be removed for the program to function correctly.
        operational_configuration: &OperationalConfiguration,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<AgentMessage<OperationalRequestMessage>, OperationalResponseMessage>>
    {
        let arc_scheduling_environment = self.scheduling_environment.clone();

        let mut operational_algorithm = OperationalAlgorithm::new(
            operational_id,
            operational_configuration,
            arc_swap_shared_solution,
        );

        for (work_order_number, work_order) in &self
            .scheduling_environment
            .lock()
            .unwrap()
            .work_orders
            .inner
        {
            for (activity_number, operation) in work_order.operations() {
                let work_order_activity = (*work_order_number, *activity_number);

                let operational_parameter_option = OperationalParameter::new(
                    operation.work_remaining().unwrap(),
                    operation.operation_analytic.preparation_time,
                );

                let operational_parameter = match operational_parameter_option {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };

                operational_algorithm
                    .insert_operational_parameter(work_order_activity, operational_parameter);
            }
        }

        let mut shared_solution_clone = (**operational_algorithm.loaded_shared_solution).clone();

        shared_solution_clone.operational.insert(
            operational_id.clone(),
            operational_algorithm.operational_solution.clone(),
        );

        operational_algorithm
            .arc_swap_shared_solution
            .0
            .store(Arc::new(shared_solution_clone));

        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();

        let mut operational_agent = Agent::new(
            asset.clone(),
            operational_id.clone(),
            arc_scheduling_environment,
            operational_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );
        assert!(!operational_agent
            .algorithm
            .operational_parameters
            .work_order_parameters
            .is_empty());

        let options = OperationalOptions::default();

        assert!(!operational_agent
            .algorithm
            .operational_parameters
            .work_order_parameters
            .is_empty());

        let thread_name = format!(
            "{} for Asset: {} for Id: {}",
            std::any::type_name::<OperationalAlgorithm>(),
            asset,
            &operational_id,
        );
        std::thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                assert!(!operational_agent
                    .algorithm
                    .operational_parameters
                    .work_order_parameters
                    .is_empty());
                operational_agent.run(options)
            })?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }
}

// This function is responsible for initializing the strategic resources
// and it is correctly based on the SchedulingEnvironment.
// QUESTION: Is this the only function that handles this?
// It is only used at creation by the agent factory. Is this correct or
// wrong? I think that is wrong. It should be handled by a different part
// of the code.
// QUESTION: Where is the file update of the WorkerEnvironment handled?
// I think that it is handled in the
// fn initialize_strategic_resources(
//     scheduling_environment: &SchedulingEnvironment,
//     start_value: Work,
// ) -> StrategicResources {
//     let mut resource_capacity: HashMap<Resources, Periods> = HashMap::new();
//     // You should now load in the technicians instead.
//     for resource in scheduling_environment
//         .worker_environment
//         .get_work_centers()
//         .iter()
//     {
//         let mut periods = HashMap::new();
//         for period in scheduling_environment
//             .time_environment
//             .strategic_periods()
//             .iter()
//         {
//             periods.insert(period.clone(), start_value.clone());
//         }
//         resource_capacity.insert(resource.clone(), Periods(periods));
//     }
//     StrategicResources::new(resource_capacity)
// }

// DEBUG: This should be removed later on.
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
            days.insert(day.clone(), start_value);
        }
        resource_capacity.insert(*resource, Days::new(days));
    }
    TacticalResources::new(resource_capacity)
}
