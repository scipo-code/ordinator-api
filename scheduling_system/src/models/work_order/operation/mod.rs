use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use shared_messages::resources::Resources;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    pub activity: u32,
    pub number: u32,
    pub work_center: Resources,
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

impl Operation {
    #[allow(dead_code)]
    pub fn new(
        activity: u32,
        number: u32,
        work_center: Resources,
        preparation_time: f64,
        work_remaining: f64,
        operating_time: f64,
        duration: u32,
        possible_start: DateTime<Utc>,
        target_finish: DateTime<Utc>,
        earliest_start_datetime: DateTime<Utc>,
        earliest_finish_datetime: DateTime<Utc>,
    ) -> Self {
        Operation {
            activity,
            number,
            work_center,
            preparation_time,
            work_remaining,
            work_performed: 0.0,
            work_adjusted: 0.0,
            operating_time,
            duration,
            possible_start,
            target_finish,
            earliest_start_datetime,
            earliest_finish_datetime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Operation;
    use chrono::Utc;
    use shared_messages::resources::Resources;

    impl Operation {
        pub fn new_test(activity: u32, work_center: Resources, work_remaining: f64) -> Self {
            Operation {
                activity,
                number: 1,
                work_center,
                preparation_time: 1.0,
                work_remaining,
                work_performed: 0.0,
                work_adjusted: 0.0,
                operating_time: 6.0,
                duration: 6,
                possible_start: Utc::now(),
                target_finish: Utc::now(),
                earliest_start_datetime: Utc::now(),
                earliest_finish_datetime: Utc::now(),
            }
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation: {} \n  work_center: {} \n  work_remaining: {} \n  operating_time: {} \n  duration: {} \n  possible_start: {} \n  target_finish: {} \n  earliest_start_datetime: {} \n  earliest_finish_datetime: {}", 
        self.number, self.work_center, self.work_remaining, self.operating_time, self.duration, self.possible_start, self.target_finish, self.earliest_start_datetime, self.earliest_finish_datetime)
    }
}
