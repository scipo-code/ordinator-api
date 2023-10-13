pub struct WorkerAgent {
    id: u32,
    agent_traits: String,
    capacity: f32,
    availability: Vec<Availability>,
    assigned: Vec<AssignedWork>,
}

struct Availability {
    start: DateTime,
    end: DateTime
}

struct AssignedWork {
    work: Work,
    start: DateTime,
    end: DateTime
}

