mod algorithm;
pub mod messages;

use std::sync::RwLockReadGuard;

use ordinator_configuration::SystemConfigurations;
use rand::rngs::StdRng;

pub struct TacticalActor(Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm>);

pub struct TacticalOptions
{
    pub number_of_removed_work_orders: usize,
    pub rng: StdRng,
}
impl From<&RwLockReadGuard<SystemConfigurations>> for TacticalOptions
{
    fn from(value: &RwLockReadGuard<SystemConfigurations>) -> Self
    {
        TacticalOptions {
            number_of_removed_work_orders: self
                .actor_configurations
                .tactical_options
                .number_of_removed_work_orders,
            rng: StdRng::from_os_rng(),
        }
    }
}
