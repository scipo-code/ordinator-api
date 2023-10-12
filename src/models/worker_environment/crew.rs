use std::collections::HashMap;
use crate::models::worker_environment::worker::Worker;


pub struct Crew {
    pub workers: HashMap<u32, Worker>,
}

impl Crew {
    pub fn new() -> Self {
        let workers = HashMap::new();
        Crew {
            workers,
        }
    }

    pub fn get_workers(&self) -> &HashMap<u32, Worker> {
        &self.workers
    }
}