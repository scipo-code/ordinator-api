mod algorithm;
mod assert_functions;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::RwLockReadGuard;

use algorithm::OperationalAlgorithm;
use algorithm::operational_solution::OperationalSolution;
use messages::OperationalRequestMessage;
use messages::OperationalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
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

impl<'a> From<RwLockReadGuard<'a, SystemConfigurations>> for OperationalOptions
{
    fn from(value: RwLockReadGuard<'a, SystemConfigurations>) -> Self
    {
        let number_of_removed_activities = value
            .actor_configurations
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
impl From<SystemConfigurations> for OperationalOptions
{
    fn from(value: SystemConfigurations) -> Self
    {
        let number_of_removed_activities = value
            .actor_configurations
            .operational_options
            .number_of_removed_work_orders;
        Self {
            number_of_removed_activities,
            rng: rand::rng(),
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
