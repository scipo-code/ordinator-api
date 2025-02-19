pub mod algorithm;
pub mod message_handlers;

use rand::rngs::StdRng;
use rand::SeedableRng;

pub struct TacticalOptions {
    number_of_removed_work_orders: usize,
    rng: StdRng,
}

impl Default for TacticalOptions {
    fn default() -> Self {
        Self {
            number_of_removed_work_orders: 25,
            rng: StdRng::from_os_rng(),
        }
    }
}
