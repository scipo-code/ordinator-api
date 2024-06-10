use serde::{Deserialize, Serialize};

use crate::models::worker_environment::worker::Worker;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Crew {
    workers: HashMap<WorkerNumber, Worker>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct WorkerNumber(pub u32);

impl Serialize for WorkerNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkerNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let worker_number_string = String::deserialize(deserializer).unwrap();
        let worker_number_primitive = worker_number_string.parse::<u32>().unwrap();
        Ok(WorkerNumber(worker_number_primitive))
    }
}

impl Crew {
    pub fn new(workers: Option<HashMap<WorkerNumber, Worker>>) -> Option<Self> {
        workers.map(|workers| Crew { workers })
    }

    pub fn get_workers(&self) -> &HashMap<WorkerNumber, Worker> {
        &self.workers
    }
}
