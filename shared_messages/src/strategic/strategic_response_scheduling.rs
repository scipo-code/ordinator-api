use serde::{Deserialize, Serialize};

use crate::models::{time_environment::period::Period, work_order::WorkOrderNumber};

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseScheduling {
    work_orders: Vec<WorkOrderNumber>,
    periods: Vec<Period>,
}

impl StrategicResponseScheduling {
    pub fn new(work_orders: Vec<WorkOrderNumber>, periods: Vec<Period>) -> Self {
        Self {
            work_orders,
            periods,
        }
    }
}
