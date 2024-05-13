use serde::Serialize;

use crate::models::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalResponseStatus {
    id: Id,
    number_of_assigned_activities: usize,
    objective: f64,
}

impl OperationalResponseStatus {
    pub fn new(id: Id, number_of_assigned_activities: usize, objective: f64) -> Self {
        Self {
            id,
            number_of_assigned_activities,
            objective,
        }
    }
}
