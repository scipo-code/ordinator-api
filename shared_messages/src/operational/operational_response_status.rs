use crate::models::worker_environment::resources::Id;

pub struct OperationalResponseStatus {
    id: Id,
    number_of_assigned_activities: u32,
    objective: f64,
}
