use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalStatusResponse {
    id: Id,
    assign_number_of_activities: usize,
    assess_number_of_activities: usize,
    unassign_number_of_activities: usize,
    objective: usize,
}

impl OperationalStatusResponse {
    pub fn new(
        id: Id,
        assign_number_of_activities: usize,
        assess_number_of_activities: usize,
        unassign_number_of_activities: usize,
        objective: usize,
    ) -> Self {
        Self {
            id,
            assign_number_of_activities,
            assess_number_of_activities,
            unassign_number_of_activities,
            objective,
        }
    }
}
