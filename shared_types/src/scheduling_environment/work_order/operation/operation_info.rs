use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo {
    number: NumberOfPeople,
    work_remaining: f64,
    work_performed: f64,
    work: f64,
    operating_time: f64,
}

impl OperationInfo {
    pub fn new(
        number: NumberOfPeople,
        work_remaining: f64,
        work_performed: f64,
        work: f64,
        operating_time: f64,
    ) -> Self {
        OperationInfo {
            number,
            work_remaining,
            work_performed,
            work,
            operating_time,
        }
    }

    pub fn work_remaining(&self) -> f64 {
        self.work_remaining
    }

    pub fn number(&self) -> NumberOfPeople {
        self.number
    }

    pub fn operating_time(&self) -> f64 {
        self.operating_time
    }
}

pub type NumberOfPeople = u32;
