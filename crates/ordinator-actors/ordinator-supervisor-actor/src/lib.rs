mod algorithm;
mod assert_functions;
pub mod messages;
use algorithm::supervisor_parameters::SupervisorParameters;
use anyhow::Result;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::SupervisorAlgorithm;
use algorithm::supervisor_solution::SupervisorSolution;
use arc_swap::ArcSwap;
use arc_swap::Guard;
#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;
use messages::SupervisorRequestMessage;
use messages::SupervisorResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct SupervisorActor<Ss>(
    Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
    Self: MessageHandler<Req = SupervisorRequestMessage, Res = SupervisorResponseMessage>;

impl<Ss> Deref for SupervisorActor<Ss>
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
{
    type Target =
        Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Ss> DerefMut for SupervisorActor<Ss>
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub struct SupervisorOptions {
    pub number_of_unassigned_work_orders: usize,
    pub rng: StdRng,
}

impl<'a> From<(&Guard<Arc<SystemConfigurations>>, &Id)> for SupervisorOptions {
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self {
        let number_of_unassigned_work_orders = value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .supervisors
            .iter()
            .find(|s| s.id == value.1.0)
            .unwrap()
            .supervisor_options
            .number_of_removed_work_orders;
        SupervisorOptions {
            rng: StdRng::from_os_rng(),
            number_of_unassigned_work_orders,
        }
    }
}
pub fn supervisor_factory<Ss>(
    id: Id,
    scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
    shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
    notify_orchestrator: Box<dyn OrchestratorNotifier>,
    system_configurations: Arc<ArcSwap<SystemConfigurations>>,
) -> Result<Communication<ActorMessage<SupervisorRequestMessage>, SupervisorResponseMessage>>
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution> + Send + Sync + 'static,
    SupervisorAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
        + Send
        + Sync
        + From<Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>>,
{
    Actor::<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>::builder()
        .agent_id(Id::new("SupervisorAgent", vec![], vec![id.asset().clone()]))
        .scheduling_environment(Arc::clone(&scheduling_environment_guard))
        // TODO
        // Make a builder here!
        // This is a little difficult. We would like to use the same scheduling environment
        // Why am I not allowed to propagate the error here?
        // Why is this so damn difficult for you to understand? What are you not understanding? I think
        // that taking a short break is a good idea.
        // The issue is that you do not understand `Fn` traits well enough
        .algorithm(|ab| {
            ab.id(id)
                // So this function returns a `Result`
                .arc_swap_shared_solution(shared_solution_arc_swap)
                .parameters_and_solution(
                    &system_configurations.load(),
                    &scheduling_environment_guard.lock().unwrap(),
                )
        })?
        // TODO [x]
        // These should be created in a single step
        .communication()
        .configurations(system_configurations)
        .notify_orchestrator(notify_orchestrator)
        .build()
}
