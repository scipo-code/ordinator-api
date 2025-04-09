use std::collections::HashMap;

use chrono::NaiveTime;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;

use super::availability::Availability;
use super::resources::Id;
use crate::time_environment::TimeInterval;
use crate::worker_environment::worker::Worker;

// TODO [ ]
// This should go to the `SchedulingEnvironment::worker_environment`
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AgentEnvironment
{
    // TODO [ ]
    // Rename these they have a horrible name, they have nothing to do with
    pub operational: HashMap<Id, OperationalConfigurationAll>,
    pub supervisor: HashMap<Id, SupervisorConfigurationAll>,
}

// WARN
// You should never be able to clone this.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationalConfigurationAll
{
    pub id: Id,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
}

impl OperationalConfigurationAll
{
    pub fn new(
        id: Id,
        hours_per_day: f64,
        operational_configuration: OperationalConfiguration,
    ) -> Self
    {
        Self {
            id,
            hours_per_day,
            operational_configuration,
        }
    }
}

// Ahh shit. You have to make them work with the correct data. You have to
// determine right now if you want to make something that. The way that you have
// it have is a little weird You should make a high level decision about how to
// structure the data correct.
//
// Ask God what to do here?
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OperationalConfiguration
{
    pub availability: Availability,
    pub break_interval: TimeInterval,
    pub off_shift_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

impl OperationalConfiguration
{
    pub fn new(
        availability: Availability,
        break_interval: TimeInterval,
        off_shift_interval: TimeInterval,
        toolbox_interval: TimeInterval,
    ) -> Self
    {
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
pub struct SupervisorConfigurationAll
{
    pub id: Id,
    // FIX
    // This information is found in two different places. That is an
    // error that has to be fixed.
    number_of_supervisor_periods: u64,
}

impl SupervisorConfigurationAll
{
    pub fn new(id: Id, number_of_supervisor_periods: u64) -> Self
    {
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
pub struct Crew
{
    workers: HashMap<WorkerNumber, Worker>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct WorkerNumber(pub u32);

impl Serialize for WorkerNumber
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkerNumber
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let worker_number_string = String::deserialize(deserializer).unwrap();
        let worker_number_primitive = worker_number_string.parse::<u32>().unwrap();
        Ok(WorkerNumber(worker_number_primitive))
    }
}

impl Crew
{
    pub fn new(workers: Option<HashMap<WorkerNumber, Worker>>) -> Option<Self>
    {
        workers.map(|workers| Crew { workers })
    }

    pub fn get_workers(&self) -> &HashMap<WorkerNumber, Worker>
    {
        &self.workers
    }
}
