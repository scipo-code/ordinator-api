use crate::models::period::Period;
use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct OrderPeriod {
    pub period: Period,  // Assuming Period is another struct you've defined.
    work_order_number: u32,
}

impl OrderPeriod {
    pub fn new(period: Period, work_order_number: u32) -> OrderPeriod {
        OrderPeriod { period: period, work_order_number: work_order_number }
    }
}

impl fmt::Display for OrderPeriod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OrdersPeriod: {{ period: {}, work_orders: {} }}", self.period.get_string(), self.work_order_number)
    }
}