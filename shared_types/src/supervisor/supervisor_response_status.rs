use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::Resources;

use super::SupervisorObjectiveValue;

#[derive(Serialize)]
pub struct SupervisorResponseStatus {
    supervisor_resource: Vec<Resources>,
    delegated_work_order_activities: usize,
    objective: SupervisorObjectiveValue,
}

impl SupervisorResponseStatus {
    pub fn new(
        main_work_center: Vec<Resources>,
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
