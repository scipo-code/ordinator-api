pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use rand::rngs::StdRng;
use rand::SeedableRng;

#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;

pub struct SupervisorOptions {
    pub number_of_unassigned_work_orders: usize,
    pub rng: StdRng,
}

