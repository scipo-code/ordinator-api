pub mod algorithm;
mod assert_functions;
pub mod messages;

use algorithm::OperationalNonProductive;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::ActorFactory;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::OperationalAlgorithm;
use algorithm::operational_parameter::OperationalParameters;
use algorithm::operational_solution::OperationalSolution;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use messages::OperationalRequestMessage;
use messages::OperationalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::rng;

// You are beginning to see the truth. That there are no shortcuts
// to be made here and no.
pub struct OperationalActor<Ss>(
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>,
)
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution>,
    Self: MessageHandler<Req = OperationalRequestMessage, Res = OperationalResponseMessage>;

pub struct OperationalOptions {
    pub number_of_removed_activities: usize,
    pub rng: StdRng,
}

// I this that this is not a good idea for the design of the system. There are
// some serious issues here with the architecture. The scheduling environment
// is growing very big and that is a good thing. It is becoming more database
// like and that is also a good thing for this kind of system.
impl From<(&Guard<Arc<SystemConfigurations>>, &Id)> for OperationalOptions {
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self {
        let number_of_removed_activities = value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .operational
            .iter()
            .find(|e| e.id == value.1.0)
            .unwrap()
            .operational_options
            .number_of_removed_work_orders;
        OperationalOptions {
            rng: StdRng::from_os_rng(),
            number_of_removed_activities,
        }
    }
}
// impl Default for OperationalOptions {
//     fn default() -> Self {
//         Self {
//             number_of_removed_activities: 50,
//             rng: StdRng::from_os_rng(),
//         }
//     }
// }
//
impl From<(SystemConfigurations, &Asset, &Id)> for OperationalOptions {
    fn from(value: (SystemConfigurations, &Asset, &Id)) -> Self {
        let number_of_removed_activities = value
            .0
            .actor_specification
            .get(value.1)
            .unwrap()
            .operational
            .iter()
            .find(|e| e.id == value.2.0)
            .unwrap()
            .operational_options
            .number_of_removed_work_orders;
        OperationalOptions {
            rng: StdRng::from_os_rng(),
            number_of_removed_activities,
        }
    }
}
impl<Ss> Deref for OperationalActor<Ss>
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution>,
{
    type Target =
        Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Ss> DerefMut for OperationalActor<Ss>
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct OperationalApi {}

impl<Ss> ActorFactory<Ss> for OperationalApi
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution> + Send + Sync + 'static,
{
    type Communication =
        Communication<ActorMessage<OperationalRequestMessage>, OperationalResponseMessage>;

    fn construct_actor(
        id: Id,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Box<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    ) -> Result<Self::Communication>
    where
        Ss: SystemSolutionTrait<Operational = OperationalSolution> + Send + Sync + 'static,
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
}
