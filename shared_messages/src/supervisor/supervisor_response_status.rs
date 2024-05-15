use serde::Serialize;

use crate::models::worker_environment::resources::{Id, MainResources};

#[derive(Serialize)]
pub struct SupervisorResponseStatus {
    main_work_center: MainResources,
    assigned_work_orders: usize,
    objective: f64,
}

impl SupervisorResponseStatus {
    pub fn new(
        main_work_center: MainResources,
        assigned_work_orders: usize,
        objective: f64,
    ) -> Self {
        Self {
            main_work_center,
            assigned_work_orders,
            objective,
        }
    }
}
