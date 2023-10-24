use chrono::{DateTime, Utc};

pub struct WorkerAgent {
    id: u32,
    agent_traits: String,
    capacity: f32,
    availability: Vec<Availability>,
    assigned: Vec<AssignedWork>,
}

struct Availability {
    start: DateTime<Utc>,
    end: DateTime<Utc>
}

struct AssignedWork {
    work: u32,
    start: DateTime<Utc>,
    end: DateTime<Utc>
}

