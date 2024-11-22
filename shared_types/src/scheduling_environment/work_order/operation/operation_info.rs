use serde::Deserialize;
use serde::Serialize;

use super::Work;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo {
    pub number: NumberOfPeople,
    pub work_remaining: Option<Work>,
    pub work_actual: Option<Work>,
    pub work: Option<Work>,
    pub operating_time: Option<Work>,
}

impl OperationInfo {
    pub fn new(
        number: NumberOfPeople,
        work_remaining: Option<Work>,
        work_actual: Option<Work>,
        work: Option<Work>,
        operating_time: Option<Work>,
    ) -> Self {
        assert!(operating_time != Some(Work::from(0.0)));
        OperationInfo {
            number,
            work_remaining,
            work_actual,
            work,
            operating_time,
        }
    }

    pub fn work_remaining(&self) -> &Option<Work> {
        &self.work_remaining
    }

    pub fn number(&self) -> NumberOfPeople {
        self.number
    }

    pub fn operating_time(&self) -> &Option<Work> {
        &self.operating_time
    }
}

pub type NumberOfPeople = u64;
