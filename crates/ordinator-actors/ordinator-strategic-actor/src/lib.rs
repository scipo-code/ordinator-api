pub mod algorithm;
//
// What
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::StrategicAlgorithm;
use algorithm::strategic_parameters::StrategicParameters;
use algorithm::strategic_solution::StrategicSolution;
use anyhow::Result;
use arc_swap::ArcSwap;
use messages::StrategicRequestMessage;
use messages::StrategicResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::AlgorithmBuilder;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use priority_queue::PriorityQueue;

pub struct StrategicActor<Ss>(
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>,
)
where
    Ss: SystemSolutionTrait<Strategic = StrategicSolution>,
    Self: MessageHandler<Req = StrategicRequestMessage, Res = StrategicResponseMessage>;

impl<Ss> Deref for StrategicActor<Ss>
where
    Ss: SystemSolutionTrait<Strategic = StrategicSolution>,
{
    type Target = Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for StrategicActor<Ss>
where
    Ss: SystemSolutionTrait<Strategic = StrategicSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

type Type<Ss> = AlgorithmBuilder<
    StrategicSolution,
    StrategicParameters,
    PriorityQueue<WorkOrderNumber, u64>,
    Ss,
>;

// This function can only work with the 'SharedSolutionTrait'.
// You cannot
// TODO [ ]
// This should be a trait that should be implemented instead. But first make the
// function work again. I think that is the best approach here.
//
// QUESTION [ ]
// What the the stance on the 'Configuration'
// Could the configuration go into the `factory`? I think that
// this is the the case. The problem is that the ActorSpecification
// should not. They could come from the `SchedulingEnvironment` I
// think that the best approach is to make something that is more
// Having the configuration centralized is a good idea. I think that
// refactoring it after this works is a better option.
pub struct StrategicApi {}
impl<Ss> ActorFactory<Ss> for StrategicApi
where
    Ss: SystemSolutionTrait<Strategic = StrategicSolution> + Send + Sync + 'static,
{
    type Communication =
        Communication<ActorMessage<StrategicRequestMessage>, StrategicResponseMessage>;

    fn construct_actor(
        id: Id,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Arc<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    ) -> Result<<Self as ActorFactory<Ss>>::Communication>
    where
        Ss: SystemSolutionTrait<Strategic = StrategicSolution> + Send + Sync + 'static,
        StrategicAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
            + Send
            + Sync
            + From<
                Algorithm<
                    StrategicSolution,
                    StrategicParameters,
                    PriorityQueue<WorkOrderNumber, u64>,
                    Ss,
                >,
            >,
    {
        Actor::<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>::builder(
        )
        .agent_id(id.clone())
        .scheduling_environment(Arc::clone(&scheduling_environment_guard))
        .algorithm(|ab| {
            ab.id(id)
                // So this function returns a `Result`
                .arc_swap_shared_solution(shared_solution_arc_swap)
                .parameters_and_solution(&scheduling_environment_guard.lock().unwrap())
        })?
        .communication()
        .configurations(system_configurations)
        .notify_orchestrator(notify_orchestrator)
        .build()
    }
}

#[cfg(test)]
mod tests
{
    use ordinator_scheduling_environment::work_order::WorkOrder;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    use ordinator_scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
    use ordinator_scheduling_environment::worker_environment::resources::Resources;

    #[test]
    fn test_extract_state_to_scheduler_overview()
    {
        WorkOrder::builder(WorkOrderNumber(2100000001))
            .operations_builder(10, Resources::MtnMech, |e| {
                e.operation_info(|e| e.work_remaining(1.0))
                    .unloading_point(UnloadingPoint::default())
            })
            .operations_builder(20, Resources::MtnMech, |e| {
                e.operation_info(|e| e.work_remaining(1.0))
                    .unloading_point(UnloadingPoint::default())
            })
            .operations_builder(30, Resources::MtnMech, |e| {
                e.operation_info(|e| e.work_remaining(1.0))
                    .unloading_point(UnloadingPoint::default())
            })
            .build();
    }
} // Crucial note, all algorithm tests have to be made in the integration tests
