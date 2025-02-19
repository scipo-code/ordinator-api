use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::{self, Formatter};

use crate::scheduling_environment::worker_environment::availability::Availability;

#[derive(Serialize, Deserialize)]
enum AssignedOrder {
    OrderInt(i32),
    None,
}

#[derive(Serialize, Deserialize)]
enum AssignedActivity {
    ActivityInt(i32),
    None,
}

#[derive(Serialize, Deserialize)]
enum AssignedTime {
    TimeFloat(f64),
    None,
}

#[derive(Serialize, Deserialize)]
struct AssignedWork {
    order: AssignedOrder,
    activity: AssignedActivity,
    time: AssignedTime,
}

#[derive(Serialize, Deserialize)]
pub struct Worker {
    name: String,
    id_worker: i32,
    capacity: f64,
    trait_: String,
    availabilities: Vec<Availability>,
    assigned_activities: Vec<AssignedWork>,
}

impl Debug for Worker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Worker")
            .field("name", &self.name)
            .field("id", &self.id_worker)
            .field("capacity", &self.capacity)
            .field("trait_", &self.trait_)
            .field("availabilities", &self.availabilities.len())
            .field("assigned_activities", &self.assigned_activities.len())
            .finish()
    }
}
