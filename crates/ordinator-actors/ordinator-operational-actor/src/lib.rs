mod algorithm;
mod assert_functions;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use algorithm::OperationalAlgorithm;
use algorithm::operational_solution::OperationalSolution;
use arc_swap::Guard;
use messages::OperationalRequestMessage;
use messages::OperationalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::rng;
use rand::rngs::ThreadRng;

// You are beginning to see the truth. That there are no shortcuts
// to be made here and no.
pub struct OperationalActor<Ss>(
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Operational = OperationalSolution>,
    Self: MessageHandler<Req = OperationalRequestMessage, Res = OperationalResponseMessage>;

pub struct OperationalOptions
{
    pub number_of_removed_activities: usize,
    pub rng: ThreadRng,
}

// I this that this is not a good idea for the design of the system. There are
// some serious issues here with the architecture. The scheduling environment
// is growing very big and that is a good thing. It is becoming more database
// like and that is also a good thing for this kind of system.
impl<'a> From<(&Guard<Arc<SystemConfigurations>>, &Id)> for OperationalOptions
{
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self
    {
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
            rng: rng(),
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
impl From<(SystemConfigurations, &Asset, &Id)> for OperationalOptions
{
    fn from(value: (SystemConfigurations, &Asset, &Id)) -> Self
    {
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
            rng: rng(),
            number_of_removed_activities,
        }
    }
}
impl<Ss> Deref for OperationalActor<Ss>
where
    Ss: SharedSolutionTrait<Operational = OperationalSolution>,
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
    Ss: SharedSolutionTrait<Operational = OperationalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}
