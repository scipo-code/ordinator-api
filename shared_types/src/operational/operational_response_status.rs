use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalStatusResponse {
    id: Id,
    assign_number_of_activities: u64,
    assess_number_of_activities: u64,
    unassign_number_of_activities: u64,
    objective: u64,
}

impl OperationalStatusResponse {
    pub fn new(
        id: Id,
        assign_number_of_activities: u64,
        assess_number_of_activities: u64,
        unassign_number_of_activities: u64,
        objective: u64,
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
