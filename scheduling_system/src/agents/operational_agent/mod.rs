pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use rand::{rngs::StdRng, SeedableRng};
use shared_types::operational::OperationalConfiguration;

pub struct OperationalOptions {
    number_of_activities: usize,
    rng: StdRng,
}

impl Default for OperationalOptions {
    fn default() -> Self {
        Self {
            number_of_activities: 50,
            rng: StdRng::from_os_rng(),
        }
    }
}
