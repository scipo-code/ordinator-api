use serde::{Deserialize, Serialize};

use ordinator_scheduling_environment::time_environment::period::Period;

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseScheduling {
    work_orders: usize,
    periods: Period,
}

impl StrategicResponseScheduling {
    pub fn new(number_of_work_orders_changed: usize, period: Period) -> Self {
        Self {
            work_orders: number_of_work_orders_changed,
            periods: period,
        }
    }
}
