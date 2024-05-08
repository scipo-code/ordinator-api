use serde::{Deserialize, Serialize};

use crate::models::worker_environment::worker::Worker;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Crew {
    workers: HashMap<u32, Worker>,
}

impl Crew {
    pub fn new(workers: Option<HashMap<u32, Worker>>) -> Option<Self> {
        workers.map(|workers| Crew { workers })
    }

    pub fn get_workers(&self) -> &HashMap<u32, Worker> {
        &self.workers
    }
}
