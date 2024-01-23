use serde::Deserialize;
use std::fmt::Debug;
use std::fmt::{self, Formatter};

use crate::models::worker_environment::availability::Availability;

#[allow(dead_code)]
#[derive(Deserialize)]
enum AssignedOrder {
    OrderInt(i32),
    None,
}

#[allow(dead_code)]
#[derive(Deserialize)]
enum AssignedActivity {
    ActivityInt(i32),
    None,
}
#[allow(dead_code)]
#[derive(Deserialize)]
enum AssignedTime {
    TimeFloat(f64),
    None,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct AssignedWork {
    order: AssignedOrder,
    activity: AssignedActivity,
    time: AssignedTime,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Worker {
    name: String,
    id: i32,
    capacity: f64,
    trait_: String,
    availabilities: Vec<Availability>,
    assigned_activities: Vec<AssignedWork>,
}

impl Debug for Worker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Worker")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("capacity", &self.capacity)
            .field("trait_", &self.trait_)
            .field("availabilities", &self.availabilities.len())
            .field("assigned_activities", &self.assigned_activities.len())
            .finish()
    }
}
