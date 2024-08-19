use serde::Serialize;

use crate::scheduling_environment::worker_environment::resources::MainResources;

#[derive(Serialize)]
pub struct SupervisorResponseStatus {
    main_work_center: MainResources,
    delegated_work_order_activities: usize,
    objective: f64,
}

impl SupervisorResponseStatus {
    pub fn new(
        main_work_center: MainResources,
        delegated_work_order_activities: usize,
        objective: f64,
    ) -> Self {
        Self {
            main_work_center,
            delegated_work_order_activities,
            objective,
        }
    }
}
