use serde::Deserialize;
use serde::Serialize;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo {
    number: u32,
    work_remaining: f64,
    work_performed: f64,
    work_adjusted: f64,
    operating_time: f64,
}

impl OperationInfo {
    pub fn new(
        number: u32,
        work_remaining: f64,
        work_performed: f64,
        work_adjusted: f64,
        operating_time: f64,
    ) -> Self {
        OperationInfo {
            number,
            work_remaining,
            work_performed,
            work_adjusted,
            operating_time,
        }
    }

    pub fn work_remaining(&self) -> f64 {
        self.work_remaining
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn operating_time(&self) -> f64 {
        self.operating_time
    }
}
