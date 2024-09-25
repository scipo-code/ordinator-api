use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalStatusResponse {
    id: Id,
    total_number_of_activities: usize,
    access_number_of_activities: usize,
    assign_number_of_activities: usize,
    objective: usize,
}

impl OperationalStatusResponse {
    pub fn new(
        id: Id,
        total_number_of_activities: usize,
        access_number_of_activities: usize,
        assign_number_of_activities: usize,
        objective: usize,
    ) -> Self {
        Self {
            id,
            total_number_of_activities,
            access_number_of_activities,
            assign_number_of_activities,
            objective,
        }
    }
}
