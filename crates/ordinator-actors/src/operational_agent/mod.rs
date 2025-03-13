pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use rand::rngs::StdRng;

pub struct OperationalOptions {
    pub number_of_removed_activities: usize,
    pub rng: StdRng,
}

// impl Default for OperationalOptions {
//     fn default() -> Self {
//         Self {
//             number_of_removed_activities: 50,
//             rng: StdRng::from_os_rng(),
//         }
//     }
// }
