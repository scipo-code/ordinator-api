pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use std::sync::RwLockReadGuard;

#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;
use ordinator_configuration::SystemConfigurations;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct SupervisorOptions
{
    pub number_of_unassigned_work_orders: usize,
    pub rng: StdRng,
}

impl From<&RwLockReadGuard<SystemConfigurations>> for SupervisorOptions
{
    fn from(value: &RwLockReadGuard<SystemConfigurations>) -> Self
    {
        let number_of_unassigned_work_orders = self
            .actor_configurations
            .supervisor_options
            .number_of_removed_work_orders;
        SupervisorOptions {
            rng: StdRng::from_os_rng(),
            number_of_unassigned_work_orders,
        }
    }
}
