use crate::models::worker_environment::availability::Availability;


enum AssignedOrder {
    OrderInt(i32),
    None,
}

enum AssignedActivity {
    ActivityInt(i32),
    None,
}

enum AssignedTime {
    TimeFloat(f64),
    None,
}
struct AssignedWork {
    order: AssignedOrder,
    activity: AssignedActivity,
    time: AssignedTime,
}

pub struct Worker {
    name: String,
    id: i32,
    capacity: f64,
    trait_: String,  // Renamed to trait_ since 'trait' is a reserved keyword in Rust.
    availabilities: Vec<Availability>,  // Assuming Availability is another struct you've defined.
    assigned_activities: Vec<AssignedWork>,
}

impl Worker {
    pub fn get_trait(&self) -> &String {
        &self.trait_
    }

    pub fn get_capacity(&self) -> &f64 {
        &self.capacity
    }
}