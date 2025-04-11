mod algorithm;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use algorithm::StrategicAlgorithm;
use algorithm::strategic_solution::StrategicSolution;
use arc_swap::Guard;
use messages::StrategicRequestMessage;
use messages::StrategicResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::time_environment::MaterialToPeriod;
use ordinator_scheduling_environment::work_order::WorkOrderConfigurations;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::SeedableRng;
use rand::rngs::StdRng;

// I really do not understand what the best approach is for going forward here
// I think that I need to understand how the whole system should be built. The
// Issue is that I do not know how the traits affect each other in the code
// and how generics play into the large structure
// How is it that these components actually work? You need to understand this
// it is like the whole system is flawed and there is nothing that you can do
// about it as you do not understand it.
// QUESTION
// What should you do to fix this? The best option is to figure it out
// completely now so that you never need to have this dilemma ever again. Make a
// test project What should this test project determine? I think that it should
// determine what What is the actual problem here? The issue is that you need a
// `contracts` module to hold all the different types of the
// I think that this is a case of you using the wrong level of abstraction. I do
// not think that there is a better way than to simply make the differnt
// message types and then implement the `From` trait on all of them. What other
// thing could be done here? I am really not sure. You know what to do here
// the issue is that you simply do not have the persistence to actually do it.
pub struct StrategicActor<Ss>(
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Strategic = StrategicSolution>,
    // I do not understand the relation, here. The issue is that I think that the
    // primary goal is to make everything generic. But it is not. You should understand
    // this. Not go quickly go the fact.
    // I think that ideally we should work with the fact that the code could work with
    // the... There is only one thing to do here and that is to really understand the
    // process of making this. You brain want to cycle and panic, that is not helpful
    // here.
    // TODO
    // Here you want the code to function on the
    Self: MessageHandler<
            Req = StrategicRequestMessage,
            Res = StrategicResponseMessage<T: ObjectiveValue>,
        >;

#[derive(Debug)]
pub struct StrategicOptions
{
    pub number_of_removed_work_order: usize,
    pub rng: StdRng,
    pub urgency_weight: usize,
    pub resource_penalty_weight: usize,
    pub clustering_weight: usize,
    pub work_order_configurations: WorkOrderConfigurations,
    pub material_to_period: MaterialToPeriod,
}
impl From<(&Guard<Arc<SystemConfigurations>>, &Id)> for StrategicOptions
{
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self
    {
        let strategic_option_config = &value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .strategic
            .strategic_options_config;

        let number_of_removed_work_order = strategic_option_config.number_of_removed_work_orders;
        let urgency_weight = strategic_option_config.urgency_weight;
        let resource_penalty_weight = strategic_option_config.resource_penalty_weight;
        let clustering_weight = strategic_option_config.clustering_weight;
        let work_order_configurations = value.0.work_order_configurations.clone();

        let material_to_period = value.0.material_to_period.clone();

        let rng = StdRng::from_os_rng();
        // QUESTION [ ]
        // _Should this field be private or public?_
        //
        // You should provide an ID here to solve this problem.
        // You should always make configuration fields private. That is
        // the best way of working with the data.
        StrategicOptions {
            number_of_removed_work_order,
            rng,
            urgency_weight,
            resource_penalty_weight,
            clustering_weight,
            work_order_configurations,
            material_to_period,
        }
    }
}
impl From<(SystemConfigurations, &Id)> for StrategicOptions
{
    fn from(value: (SystemConfigurations, &Id)) -> Self
    {
        let strategic_option_config = &value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .strategic
            .strategic_options_config;

        let number_of_removed_work_order = strategic_option_config.number_of_removed_work_orders;
        let urgency_weight = strategic_option_config.urgency_weight;
        let resource_penalty_weight = strategic_option_config.resource_penalty_weight;
        let clustering_weight = strategic_option_config.clustering_weight;
        let work_order_configurations = value.0.work_order_configurations;

        let material_to_period = value.0.material_to_period;

        let rng = StdRng::from_os_rng();
        // QUESTION [ ]
        // _Should this field be private or public?_
        //
        // You should provide an ID here to solve this problem.
        // You should always make configuration fields private. That is
        // the best way of working with the data.
        StrategicOptions {
            number_of_removed_work_order,
            rng,
            urgency_weight,
            resource_penalty_weight,
            clustering_weight,
            work_order_configurations,
            material_to_period,
        }
    }
}
impl<Ss> Deref for StrategicActor<Ss>
where
    Ss: SharedSolutionTrait<Strategic = StrategicSolution>,
{
    type Target = Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for StrategicActor<Ss>
where
    Ss: SharedSolutionTrait<Strategic = StrategicSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
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
