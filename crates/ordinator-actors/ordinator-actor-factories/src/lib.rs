use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
// These are mostly build dependencies and should therefore not be found inside of the
// `orchestrator`. The question then becomes what we should do about the building of the
// `actors`
//
// I think that you should go to the gym now. There is an issue here in that
// I do not know what the best way to proceed is for the different.
use ordinator_actors::Actor;
use ordinator_actors::ActorMessage;
use ordinator_actors::Algorithm;
use ordinator_actors::OperationalSolution;
use ordinator_actors::SharedSolution;
use ordinator_actors::StrategicSolution;
use ordinator_actors::SupervisorSolution;
use ordinator_actors::TacticalSolution;
use ordinator_actors::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actors::traits::Parameters;
use ordinator_contracts::operational::OperationalRequestMessage;
use ordinator_contracts::operational::OperationalResponseMessage;
use ordinator_contracts::strategic::StrategicRequestMessage;
use ordinator_contracts::strategic::StrategicResponseMessage;
use ordinator_contracts::supervisor::SupervisorRequestMessage;
use ordinator_contracts::supervisor::SupervisorResponseMessage;
use ordinator_contracts::tactical::TacticalRequestMessage;
use ordinator_contracts::tactical::TacticalResponseMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::NotifyOrchestrator;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;

#[derive(Debug, Clone)]
pub struct AgentFactory {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

// TODO [ ]
// Move every single function into the Agents themselves and have the orchestrator input
// them. This means that almost everything should be made as "non-pub". This is the crucial
// lesson learned from 100s of failures.
// You should test a single instance of the factory. That is the most crucial aspect. Make a single
// function.
impl AgentFactory {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        AgentFactory {
            scheduling_environment,
        }
    }

    // This should be moved to the `Orchestrator`
    pub fn create_shared_solution_arc_swap() -> Arc<ArcSwap<SharedSolution>> {
        let shared_solution_arc_swap = SharedSolution::default();

        Arc::new(ArcSwap::from(Arc::new(shared_solution_arc_swap)))
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

        let mut tactical_agent = Actor::new(
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
        let mut supervisor_agent = Actor::new(
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

        let mut operational_agent = Actor::new(
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
