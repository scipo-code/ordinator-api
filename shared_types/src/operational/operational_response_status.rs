use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalStatusResponse {
    id: Id,
    number_of_assigned_activities: usize,
    objective: usize,
}

impl OperationalStatusResponse {
    pub fn new(id: Id, number_of_assigned_activities: usize, objective: usize) -> Self {
        Self {
            id,
            number_of_assigned_activities,
            objective,
        }
    }
}
