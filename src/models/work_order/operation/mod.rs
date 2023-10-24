use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Operation {
    pub activity: u32,
    pub number: u32,
    pub work_center: String,  
    pub preparation_time: f64,
    pub work_remaining: f64,
    pub work_performed: f64,
    pub work_adjusted: f64,
    pub operating_time: f64,
    pub duration: u32,
    pub possible_start: DateTime<Utc>,
    pub target_finish: DateTime<Utc>,
    pub earliest_start_datetime: DateTime<Utc>,
    pub earliest_finish_datetime: DateTime<Utc>,
}