pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use std::sync::RwLockReadGuard;

use rand::rngs::StdRng;

pub struct OperationalOptions {
    pub number_of_removed_activities: usize,
    pub rng: StdRng,
}

impl From<&RwLockReadGuard<SystemConfiguration>> for OperationalOptions {
    fn from(value: &RwLockReadGuard<SystemConfiguration>) -> Self {
        let number_of_removed_activities = self
            .actor_configurations
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
