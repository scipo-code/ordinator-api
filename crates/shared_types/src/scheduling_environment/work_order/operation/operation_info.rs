use serde::Deserialize;
use serde::Serialize;

use super::Work;

pub type NumberOfPeople = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo {
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_actual: Work,
    pub work: Work,
    pub operating_time: Work,
}

// Good! The fields should be optional in the OperationInfoBuilder, not the OperationInfo
pub struct OperationInfoBuilder {
    number: Option<NumberOfPeople>,
    work_remaining: Option<Work>,
    work_actual: Option<Work>,
    work: Option<Work>,
    operating_time: Option<Work>,
}

impl OperationInfo {
    pub fn builder() -> OperationInfoBuilder {
        OperationInfoBuilder {
            number: todo!(),
            work_remaining: todo!(),
            work_actual: todo!(),
            work: todo!(),
            operating_time: todo!(),
        }
    }
}

impl OperationInfoBuilder {
    pub fn build(self) -> OperationInfo {
        OperationInfo {
            number: self.number.unwrap_or(1),
            work_remaining: self.work_remaining.unwrap_or_default(),
            work_actual: self.work_actual.unwrap_or_default(),
            work: self.work.unwrap_or_default(),
            // FIX [ ]
            // The default operating time should come from the
            operating_time: self.operating_time.unwrap_or_default(),
        }
    }

    pub fn number(mut self, number: NumberOfPeople) -> Self {
        self.number = Some(number);
        self
    }
    pub fn work_remaining(mut self, work_remaining: f64) -> Self {
        self.work_remaining = Some(Work::from(work_remaining));
        self
    }
    pub fn work_actual(mut self, work_actual: f64) -> Self {
        self.work_actual = Some(Work::from(work_actual));
        self
    }
    pub fn work(mut self, work: f64) -> Self {
        self.work = Some(Work::from(work));
        self
    }
    pub fn operating_time(mut self, operating_time: f64) -> Self {
        self.operating_time = Some(Work::from(operating_time));
        self
    }
}
