pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use rand::rngs::StdRng;
use rand::SeedableRng;

#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;

pub struct SupervisorOptions {
    number_of_unassigned_work_orders: usize,
    rng: StdRng,
}

impl Default for SupervisorOptions {
    fn default() -> Self {
        Self {
            number_of_unassigned_work_orders: 25,
            rng: StdRng::from_os_rng(),
        }
    }
}
