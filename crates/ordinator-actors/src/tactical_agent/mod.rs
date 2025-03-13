pub mod algorithm;
pub mod message_handlers;

use rand::rngs::StdRng;

pub struct TacticalOptions {
    pub number_of_removed_work_orders: usize,
    pub rng: StdRng,
}
