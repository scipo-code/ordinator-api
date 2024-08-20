use serde::Deserialize;
use serde::Serialize;

use super::Work;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo {
    number: NumberOfPeople,
    work_remaining: Work,
    work_performed: Work,
    work: Work,
    operating_time: Work,
}

impl OperationInfo {
    pub fn new(
        number: NumberOfPeople,
        work_remaining: Work,
        work_performed: Work,
        work: Work,
        operating_time: Work,
    ) -> Self {
        OperationInfo {
            number,
            work_remaining,
            work_performed,
            work,
            operating_time,
        }
    }

    pub fn work_remaining(&self) -> &Work {
        &self.work_remaining
    }

    pub fn number(&self) -> NumberOfPeople {
        self.number
    }

    pub fn operating_time(&self) -> &Work {
        &self.operating_time
    }
}

pub type NumberOfPeople = u32;
