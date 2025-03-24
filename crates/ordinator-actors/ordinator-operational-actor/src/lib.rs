pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use std::sync::RwLockReadGuard;

use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use rand::rng;
use rand::rngs::ThreadRng;

// Is this actually needed? I do not think so
// pub struct OperationalActor(
//     Actor<
//         OperationalRequestMessage,
//         OperationalResponseMessage,
//         OperationalSolution,
//         OperationalParameters,
//         OperationalNonProductive,
//         Ss,
//     >,
// );

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
