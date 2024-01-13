use crate::models::worker_environment::availability::Availability;

#[allow(dead_code)]
enum AssignedOrder {
    OrderInt(i32),
    None,
}

#[allow(dead_code)]
enum AssignedActivity {
    ActivityInt(i32),
    None,
}
#[allow(dead_code)]
enum AssignedTime {
    TimeFloat(f64),
    None,
}

#[allow(dead_code)]
struct AssignedWork {
    order: AssignedOrder,
    activity: AssignedActivity,
    time: AssignedTime,
}

#[allow(dead_code)]
pub struct Worker {
    name: String,
    id: i32,
    capacity: f64,
    trait_: String, // Renamed to trait_ since 'trait' is a reserved keyword in Rust.
    availabilities: Vec<Availability>, // Assuming Availability is another struct you've defined.
    assigned_activities: Vec<AssignedWork>,
}
