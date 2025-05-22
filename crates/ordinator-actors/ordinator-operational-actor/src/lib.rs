pub mod algorithm;
mod assert_functions;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::OperationalAlgorithm;
use algorithm::OperationalNonProductive;
use algorithm::operational_parameter::OperationalParameters;
use algorithm::operational_solution::OperationalSolution;
use anyhow::Result;
use arc_swap::ArcSwap;
use flume::Sender;
use messages::OperationalRequestMessage;
use messages::OperationalResponseMessage;
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

// You are beginning to see the truth. That there are no shortcuts
// to be made here and no.
pub struct OperationalActor<Ss>(
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
    Self: MessageHandler<Req = OperationalRequestMessage, Res = OperationalResponseMessage>;

impl<Ss> Deref for OperationalActor<Ss>
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
{
    type Target =
        Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for OperationalActor<Ss>
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct OperationalApi {}

impl<Ss> ActorFactory<Ss> for OperationalApi
where
    Ss: SystemSolutions<Operational = OperationalSolution> + Send + Sync + 'static,
{
    type Communication = Communication<OperationalRequestMessage, OperationalResponseMessage>;

    fn construct_actor(
        id: Id,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Arc<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        error_channel: Sender<ActorError>,
    ) -> Result<Self::Communication>
    where
        Ss: SystemSolutions<Operational = OperationalSolution> + Send + Sync + 'static,
        OperationalAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
            + Send
            + Sync
            + From<
                Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>,
            >,
    {
        Actor::<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>::builder()
        .agent_id(Id::new("OperationalAgent", vec![], vec![id.asset().clone()]))
        .scheduling_environment(Arc::clone(&scheduling_environment_guard))
        .algorithm(|ab| {
            ab.id(id)
                // So this function returns a `Result`
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
