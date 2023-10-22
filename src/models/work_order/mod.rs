pub mod operation;
pub mod order_dates;
pub mod order_text;
pub mod status_codes;
pub mod functional_location;
pub mod unloading_point;
pub mod revision;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::work_order::operation::Operation;
use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::revision::Revision;

#[derive(Serialize, Deserialize)]
pub enum Priority {
    IntValue(i32),
    StringValue(String),
}

#[derive(Serialize, Deserialize)]
pub struct WorkOrder {
    pub order_number: u32,
    pub fixed: bool,
    pub order_weight: u32,
    pub priority: Priority,
    pub order_work: f64,
    pub operations: HashMap<u32, Operation>,
    pub work_load: HashMap<String, f64>, 
    pub start_start: Vec<bool>,
    pub finish_start: Vec<bool>,
    pub postpone: Vec<DateTime<Utc>>,
    pub order_type: String,
    pub status_codes: StatusCodes,  
    pub order_dates: OrderDates,
    pub revision: Revision,
    pub unloading_point: UnloadingPoint, 
    pub functional_location: FunctionalLocation, 
    pub order_text: OrderText,
    pub vendor: bool,
}

impl WorkOrder {
    pub fn get_work_order_number(&self) -> u32 {
        self.order_number
    }

    pub fn insert_operation(&mut self, operation: Operation) {
        self.operations.insert(operation.activity, operation);
    }
}

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order Number: {}, \nNumber of activities: {}, \nVendor: {}, \nAWSC: {}, \nShutdown", self.order_number, self.operations.len(), self.vendor, self.status_codes.awsc)
    }
}