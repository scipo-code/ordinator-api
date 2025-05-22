pub mod algorithm;
mod assert_functions;
pub mod messages;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::SupervisorAlgorithm;
use algorithm::supervisor_parameters::SupervisorParameters;
use algorithm::supervisor_solution::SupervisorSolution;
use anyhow::Result;
use arc_swap::ArcSwap;
#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;
use flume::Sender;
use messages::SupervisorRequestMessage;
use messages::SupervisorResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorError;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;

pub struct SupervisorActor<Ss>(
    Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
    Self: MessageHandler<Req = SupervisorRequestMessage, Res = SupervisorResponseMessage>;

impl<Ss> Deref for SupervisorActor<Ss>
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    type Target =
        Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for SupervisorActor<Ss>
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

// You have to work much harder to get this going. You have to remain completely
// calm.
pub struct SupervisorApi {}

// When you do it like this you tie the code of the Ss into the type. You have
// to be aware of this for the future, but for now you simply need to get this
// working again.
impl<Ss> ActorFactory<Ss> for SupervisorApi
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution> + Send + Sync + 'static,
    SupervisorAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
        + Send
        + Sync
        + From<Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>>,
{
    type Communication = Communication<SupervisorRequestMessage, SupervisorResponseMessage>;

    fn construct_actor(
        id: Id,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Arc<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        error_channel: Sender<ActorError>,
    ) -> Result<Self::Communication>
    where
        Ss: SystemSolutions<Supervisor = SupervisorSolution> + Send + Sync + 'static,
    {
        Actor::<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>::builder()
        .agent_id(id.clone())
        .scheduling_environment(Arc::clone(&scheduling_environment_guard))
        .algorithm(|ab| {
            ab.id(id)
                // So this function returns a `Result`.
                .parameters_and_solution(
                    &scheduling_environment_guard.lock().unwrap(),
                )?
                .arc_swap_shared_solution(shared_solution_arc_swap)
        })?
        .communication(error_channel)
        .configurations(system_configurations)
        .notify_orchestrator(notify_orchestrator)
        .build()
    }
}
