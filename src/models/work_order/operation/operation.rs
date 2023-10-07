use chrono::{DateTime, Utc};

pub struct Operation {
    activity: i32,
    number: i32,
    work_center: char,  // Assuming 'Symbol' translates to a char in Rust.
    preparation_time: f64,
    work_remaining: f64,
    work_performed: f64,
    work_adjusted: f64,
    operating_time: f64,
    duration: i32,
    possible_start: f64,
    target_finish: f64,
    earliest_start_datetime: DateTime<Utc>,
    earliest_finish_datetime: DateTime<Utc>,
}