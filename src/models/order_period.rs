use crate::models::period::Period;
use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct OrderPeriod {
    period: Period,  // Assuming Period is another struct you've defined.
    work_order_number: u32,
}

impl fmt::Display for OrderPeriod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OrdersPeriod: {{ period: {}, work_orders: {} }}", self.period.get_period(), self.work_order_number)
    }
}