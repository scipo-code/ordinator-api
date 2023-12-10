use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
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




#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::Operation;


    impl Operation {
        pub fn new_test(activity: u32, work_center: String, work_remaining: f64) -> Self {

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