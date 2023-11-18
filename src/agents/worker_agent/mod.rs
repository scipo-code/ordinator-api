use chrono::{DateTime, Utc};
use actix::prelude::*;

#[allow(dead_code)]
pub struct WorkerAgent {
    id: u32,
    agent_traits: String,
    capacity: f32,
    availability: Vec<Availability>,
    assigned: Vec<AssignedWork>,
}

#[allow(dead_code)]
struct Availability {
    start: DateTime<Utc>,
    end: DateTime<Utc>
}

#[allow(dead_code)]
struct AssignedWork {
    work: u32,
    start: DateTime<Utc>,
    end: DateTime<Utc>
}


impl Actor for WorkerAgent {
    type Context = Context<Self>;
}