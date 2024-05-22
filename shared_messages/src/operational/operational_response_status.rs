use serde::Serialize;

use crate::models::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalStatusResponse {
    id: Id,
    number_of_assigned_activities: usize,
    objective: f64,
}

impl OperationalStatusResponse {
    pub fn new(id: Id, number_of_assigned_activities: usize, objective: f64) -> Self {
        Self {
            id,
            number_of_assigned_activities,
            objective,
        }
    }
}
