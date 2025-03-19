use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

#[derive(Serialize)]
pub struct OperationalResponseStatus {
    id: Id,
    assign_number_of_activities: u64,
    assess_number_of_activities: u64,
    unassign_number_of_activities: u64,
    objective: u64,
}

impl OperationalResponseStatus {
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
