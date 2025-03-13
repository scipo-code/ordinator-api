use serde::{Deserialize, Serialize};

use crate::scheduling_environment::worker_environment::worker::Worker;
use std::collections::HashMap;

// TODO [ ]
// This should go to the `SchedulingEnvironment::worker_environment`
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AgentEnvironment {
    // TODO [ ]
    // Rename these they have a horrible name, they have nothing to do with
    pub operational: HashMap<Id, OperationalConfigurationAll>,
    pub supervisor: HashMap<Id, SupervisorConfigurationAll>,
}

// WARN
// You should never be able to clone this.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationalConfigurationAll {
    pub id: Id,
    hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
}

impl OperationalConfigurationAll {
    pub fn new(
        id: Id,
        hours_per_day: f64,
        operational_configuration: OperationalConfiguration,
    ) -> Self {
        Self {
            id,
            hours_per_day,
            operational_configuration,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfigurationAll {
    id: Id,
    // FIX
    // This information is found in two different places. That is an
    // error that has to be fixed.
    number_of_supervisor_periods: u64,
}

// TODO [ ]
// I do not think that this is relevant any more. It should be deleted and
// not to be seen again.
#[derive(Serialize, Deserialize, Debug)]
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
