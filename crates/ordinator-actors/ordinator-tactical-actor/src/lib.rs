mod algorithm;
pub mod messages;

use algorithm::tactical_parameters::TacticalParameters;
use anyhow::Result;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use priority_queue::PriorityQueue;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::TacticalAlgorithm;
use algorithm::tactical_solution::TacticalSolution;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use messages::TacticalRequestMessage;
use messages::TacticalResponseMessage;
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

//TODO [ ]
// Make `TacticalAlgorithm` but... Eat first.
pub struct TacticalActor<Ss>(
    Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
    Self: MessageHandler<Req = TacticalRequestMessage, Res = TacticalResponseMessage>;

pub struct TacticalOptions {
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
    pub rng: StdRng,
}
impl From<(&Guard<Arc<SystemConfigurations>>, &Id)> for TacticalOptions {
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self {
        let tactical_options_config = &value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .tactical
            .tactical_options_config;
        TacticalOptions {
            number_of_removed_work_orders: tactical_options_config.number_of_removed_work_orders,
            rng: StdRng::from_os_rng(),
            urgency: tactical_options_config.urgency,
            resource_penalty: tactical_options_config.resource_penalty,
        }
    }
}

impl<Ss> Deref for TacticalActor<Ss>
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
{
    type Target = Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Ss> DerefMut for TacticalActor<Ss>
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub fn tactical_factory<Ss>(
    id: Id,
    scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
    shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
    notify_orchestrator: Box<dyn OrchestratorNotifier>,
    system_configurations: Arc<ArcSwap<SystemConfigurations>>,
) -> Result<Communication<ActorMessage<TacticalRequestMessage>, TacticalResponseMessage>>
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution> + Send + Sync + 'static,
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
    Actor::<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>::builder()
        .agent_id(Id::new("TacticalAgent", vec![], vec![id.asset().clone()]))
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
