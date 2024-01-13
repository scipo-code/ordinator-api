use crate::models::worker_environment::worker::Worker;
use std::collections::HashMap;

pub struct Crew {
    pub workers: HashMap<u32, Worker>,
}

impl Crew {
    pub fn new() -> Self {
        let workers = HashMap::new();
        Crew { workers }
    }
}
