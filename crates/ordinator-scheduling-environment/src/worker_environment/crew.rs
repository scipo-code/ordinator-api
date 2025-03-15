use chrono::NaiveTime;
use serde::{Deserialize, Deserializer, Serialize, de};

use crate::worker_environment::worker::Worker;
use std::collections::HashMap;

use super::{availability::Availability, resources::Id};

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
    pub hours_per_day: f64,
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
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OperationalConfiguration {
    pub availability: Availability,
    pub break_interval: TimeInterval,
    pub off_shift_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct TimeInterval {
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub start: NaiveTime,
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub end: NaiveTime,
}

fn deserialize_time_interval<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time_str: String = Deserialize::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time_str, "%H:%M:%S").map_err(de::Error::custom)
}

impl OperationalConfiguration {
    pub fn new(
        availability: Availability,
        break_interval: TimeInterval,
        off_shift_interval: TimeInterval,
        toolbox_interval: TimeInterval,
    ) -> Self {
        Self {
            availability,
            break_interval,
            off_shift_interval,
            toolbox_interval,
        }
    }
}

// What should the fields be here?
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfigurationAll {
    pub id: Id,
    // FIX
    // This information is found in two different places. That is an
    // error that has to be fixed.
    number_of_supervisor_periods: u64,
}

impl SupervisorConfigurationAll {
    pub fn new(id: Id, number_of_supervisor_periods: u64) -> Self {
        Self {
            id,
            number_of_supervisor_periods,
        }
    }
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
