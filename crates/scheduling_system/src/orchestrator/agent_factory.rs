use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use shared_types::agents::operational::{OperationalRequestMessage, OperationalResponseMessage};
use shared_types::agents::strategic::{StrategicRequestMessage, StrategicResponseMessage};
use shared_types::agents::supervisor::{SupervisorRequestMessage, SupervisorResponseMessage};
use shared_types::agents::tactical::{
    Days, TacticalRequestMessage, TacticalResources, TacticalResponseMessage,
};
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::Asset;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, MutexGuard};

use crate::agents::operational_agent::algorithm::operational_parameter::OperationalParameters;
use crate::agents::operational_agent::OperationalOptions;
use crate::agents::orchestrator::{Communication, NotifyOrchestrator};
use crate::agents::strategic_agent::algorithm::strategic_parameters::StrategicParameters;
use crate::agents::strategic_agent::StrategicOptions;
use crate::agents::supervisor_agent::algorithm::supervisor_parameters::SupervisorParameters;
use crate::agents::supervisor_agent::SupervisorOptions;
use crate::agents::tactical_agent::algorithm::tactical_parameters::TacticalParameters;
use crate::agents::tactical_agent::TacticalOptions;
use crate::agents::traits::{ActorBasedLargeNeighborhoodSearch, Parameters};
use crate::agents::{
    ActorMessage, Agent, Algorithm, AlgorithmUtils, ArcSwapSharedSolution, OperationalSolution,
    SharedSolution, Solution, StrategicSolution, SupervisorSolution, TacticalSolution,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
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

    // Okay very good! Every agent should look like this! That is important.
    pub fn build_strategic_agent(
        &self,
        asset: &Asset,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        shared_solution_arc_swap: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
        // Okay now the issue is that we have to decide what to do with the
        // general configurations. I think that this is the best approach for
        // generating the best result.
        strategic_options: StrategicOptions,
    ) -> Result<Communication<ActorMessage<StrategicRequestMessage>, StrategicResponseMessage>>
    {
        let strategic_id = Id::new("StrategicAgent", vec![], vec![asset.clone()]);

        let strategic_parameters = StrategicParameters::new(
            &strategic_id,
            strategic_options,
            scheduling_environment_guard,
        )
        .with_context(|| format!("Failed to create StrategicParameters for {}", asset))?;

        let strategic_solution = StrategicSolution::new(&strategic_parameters);

        let strategic_algorithm: Algorithm<
            StrategicSolution,
            StrategicParameters,
            // FIX
            // You do not want a priority queue you want a BinaryHeap. That is a much
            // better choice.
            priority_queue::PriorityQueue<WorkOrderNumber, u64>,
        > = Algorithm::new(
            &strategic_id,
            strategic_solution,
            strategic_parameters,
            shared_solution_arc_swap,
        );

        let arc_scheduling_environment = Arc::clone(&self.scheduling_environment);

        let (sender_to_agent, receiver_from_orchestrator): (
            std::sync::mpsc::Sender<ActorMessage<StrategicRequestMessage>>,
            std::sync::mpsc::Receiver<ActorMessage<StrategicRequestMessage>>,
        ) = sync::mpsc::channel();

        let (sender_to_orchestrator, receiver_from_agent): (
            std::sync::mpsc::Sender<std::result::Result<StrategicResponseMessage, anyhow::Error>>,
            std::sync::mpsc::Receiver<std::result::Result<StrategicResponseMessage, anyhow::Error>>,
        ) = std::sync::mpsc::channel();

        let mut strategic_agent = Agent::new(
            strategic_id,
            arc_scheduling_environment,
            strategic_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        let thread_name = format!(
            "{} for Asset: {}",
            std::any::type_name_of_val(&strategic_agent),
            asset,
        );
        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || strategic_agent.run())?;

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
    ) -> Result<Communication<ActorMessage<TacticalRequestMessage>, TacticalResponseMessage>> {
        // This is a horrible approach. You should centralize it first.

        let tactical_id = Id::new(
            &("TACTICAL".to_string() + &asset.to_string()),
            vec![],
            vec![asset.clone()],
        );

        let options = TacticalOptions::default();

        let tactical_parameters =
            TacticalParameters::new(&tactical_id, options, scheduling_environment_guard)?;

        let tactical_solution = TacticalSolution::new(&tactical_parameters);

        let tactical_algorithm = Algorithm::new(
            &tactical_id,
            tactical_solution,
            tactical_parameters,
            strategic_tactical_optimized_work_orders,
        );

        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();

        let arc_scheduling_environment = self.scheduling_environment.clone();

        let mut tactical_agent = Agent::new(
            tactical_id,
            arc_scheduling_environment,
            tactical_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        tactical_agent.algorithm.make_atomic_pointer_swap();
        let thread_name = format!(
            "{} for Asset: {}",
            std::any::type_name_of_val(&tactical_agent.algorithm),
            asset,
        );

        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || tactical_agent.run())?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }

    pub fn build_supervisor_agent(
        &self,
        asset: &Asset,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        id_supervisor: &Id,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<ActorMessage<SupervisorRequestMessage>, SupervisorResponseMessage>>
    {
        let options = SupervisorOptions::default();

        let supervisor_parameters =
            SupervisorParameters::new(id_supervisor, options, scheduling_environment_guard)?;

        let supervisor_solution = SupervisorSolution::new(&supervisor_parameters);

        let supervisor_algorithm = Algorithm::new(
            id_supervisor,
            supervisor_solution,
            supervisor_parameters,
            arc_swap_shared_solution,
        );

        let scheduling_environment = Arc::clone(&self.scheduling_environment);

        let (sender_to_agent, receiver_from_orchestrator) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent) = std::sync::mpsc::channel();
        // It is the `Algorithm` that should have the arc_swap_shared_solution, not the
        // `Agent`. The mutable requirements makes a lot of sense.
        let mut supervisor_agent = Agent::new(
            id_supervisor.clone(),
            scheduling_environment,
            supervisor_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        std::thread::Builder::new()
            .name(format!(
                "{} for Id: {}",
                std::any::type_name_of_val(&supervisor_agent),
                &id_supervisor,
            ))
            .spawn(move || supervisor_agent.run())?;

        Ok(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        })
    }

    // Should the `OperationalConfiguration` not be in the
    // `SchedulingEnvironment`? Yes I think that it should.
    // FIX
    // Move the OperationalConfiguration into the `SchedulingEnvironment`.
    // QUESTION
    // Is it a good idea that the asset is part of the ID? Yes that is a good idea!
    pub fn build_operational_agent(
        &self,
        operational_id: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<Communication<ActorMessage<OperationalRequestMessage>, OperationalResponseMessage>>
    {
        let options = OperationalOptions::default();

        let operational_parameters =
            OperationalParameters::new(operational_id, options, scheduling_environment)
                .context("Parameters could not be created")?;

        let operational_solution = OperationalSolution::new(&operational_parameters);

        let operational_algorithm = Algorithm::new(
            operational_id,
            operational_solution,
            operational_parameters,
            arc_swap_shared_solution,
        );

        let (sender_to_agent, receiver_from_orchestrator): (
            std::sync::mpsc::Sender<ActorMessage<OperationalRequestMessage>>,
            std::sync::mpsc::Receiver<ActorMessage<OperationalRequestMessage>>,
        ) = std::sync::mpsc::channel();
        let (sender_to_orchestrator, receiver_from_agent): (
            std::sync::mpsc::Sender<std::result::Result<OperationalResponseMessage, anyhow::Error>>,
            std::sync::mpsc::Receiver<
                std::result::Result<OperationalResponseMessage, anyhow::Error>,
            >,
        ) = std::sync::mpsc::channel();

        let mut operational_agent = Agent::new(
            operational_id.clone(),
            self.scheduling_environment.clone(),
            operational_algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        );

        // WARN
        // This should be the default for all builders
        // WARN
        operational_agent.algorithm.make_atomic_pointer_swap();

        // Standardize thread names.
        let thread_name = format!(
            "{} for Id: {}",
            std::any::type_name::<OperationalSolution>(),
            &operational_id,
        );

        std::thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || operational_agent.run())?;

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
