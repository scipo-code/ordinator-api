use serde::Serialize;

use ordinator_scheduling_environment::worker_environment::resources::Id;

use crate::agents::supervisor::SupervisorObjectiveValue;

#[derive(Serialize)]
pub struct SupervisorResponseStatus {
    supervisor_resource: Vec<Id>,
    delegated_work_order_activities: usize,
    objective: SupervisorObjectiveValue,
}

impl SupervisorResponseStatus {
    pub fn new(
        main_work_center: Vec<Id>,
        delegated_work_order_activities: usize,
        objective: SupervisorObjectiveValue,
    ) -> Self {
        Self {
            supervisor_resource: main_work_center,
            delegated_work_order_activities,
            objective,
        }
    }
}
