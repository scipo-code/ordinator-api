use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Throttling {
    strategic: u64,
    tactical: u64,
    supervisor: u64,
    operational: u64,
}
