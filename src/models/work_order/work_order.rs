use std::collections::HashMap;
use crate::models::work_order::operation::operation::Operation;

use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::revision::Revision;


enum Priority {
    IntValue(i32),
    StringValue(String),
}

pub struct WorkOrder {
    order_number: i32,
    fixed: bool,
    order_weight: i32,
    priority: Priority,
    order_work: f64,
    operations: Vec<Operation>,
    work_load: HashMap<char, f64>, // Assuming 'Symbol' translates to a char in Rust.
    start_start: Vec<bool>,
    finish_start: Vec<bool>,
    postpone: Vec<f64>,
    order_type: String,
    status_codes: StatusCodes,  // Assuming StatusCodesOrder is another struct.
    order_dates: OrderDates,
    revision: Revision,
    unloading_point: UnloadingPoint, // Assuming UnloadingPoint is another struct.
    functional_location: FunctionalLocation, // Assuming FunctionalLocation is another struct.
    order_text: OrderText,
    vendor: bool,
}