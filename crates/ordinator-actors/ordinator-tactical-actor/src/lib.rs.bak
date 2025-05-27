pub mod algorithm;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::TacticalAlgorithm;
use algorithm::tactical_parameters::TacticalParameters;
use algorithm::tactical_solution::TacticalSolution;
use anyhow::Result;
use arc_swap::ArcSwap;
use flume::Sender;
use messages::TacticalRequestMessage;
use messages::TacticalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use priority_queue::PriorityQueue;

pub struct TacticalActor<Ss>(
    Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Tactical = TacticalSolution>,
    Self: MessageHandler<Req = TacticalRequestMessage, Res = TacticalResponseMessage>;

impl<Ss> Deref for TacticalActor<Ss>
where
    Ss: SystemSolutions<Tactical = TacticalSolution>,
{
    type Target = Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for TacticalActor<Ss>
where
    Ss: SystemSolutions<Tactical = TacticalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct TacticalApi {}

impl<Ss> ActorFactory<Ss> for TacticalApi
where
    Ss: SystemSolutions<Tactical = TacticalSolution> + Send + Sync + 'static,
    TacticalAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
        + Send
        + Sync
        + From<
            Algorithm<
                TacticalSolution,
                TacticalParameters,
                PriorityQueue<WorkOrderNumber, u64>,
                Ss,
            >,
        >,
{
    type Communication = Communication<TacticalRequestMessage, TacticalResponseMessage>;

    fn construct_actor(
        id: Id,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Arc<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<Self::Communication>
    {
        Actor::<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>::builder()
            .agent_id(Id::new("TacticalAgent", vec![], vec![id.asset().clone()]))
            .scheduling_environment(Arc::clone(&scheduling_environment_guard))
            // TODO
            // Make a builder here!
            // This is a little difficult. We would like to use the same scheduling environment
            // Why am I not allowed to propagate the error here?
            // Why is this so damn difficult for you to understand? What are you not understanding?
            // I think that taking a short break is a good idea.
            // The issue is that you do not understand `Fn` traits well enough
            .algorithm(|ab| {
                ab.id(id)
                    // So this function returns a `Result`
                    .parameters_and_solution(&scheduling_environment_guard.lock().unwrap())?
                    .arc_swap_shared_solution(shared_solution_arc_swap)
            })?
            // TODO [x]
            // These should be created in a single step
            .communication(error_channel)
            .configurations(system_configurations)
            .notify_orchestrator(notify_orchestrator)
            .build()
    }
}
