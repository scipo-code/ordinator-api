use serde::Deserialize;
use serde::Serialize;

use super::Work;

pub type NumberOfPeople = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationInfo
{
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_actual: Work,
    pub work: Work,
}

// Good! The fields should be optional in the OperationInfoBuilder, not the
// OperationInfo.
pub struct OperationInfoBuilder
{
    number: Option<NumberOfPeople>,
    work_remaining: Option<Work>,
    work_actual: Option<Work>,
    work: Option<Work>,
}

impl OperationInfo
{
    pub fn builder() -> OperationInfoBuilder
    {
        OperationInfoBuilder {
            number: None,
            work_remaining: None,
            work_actual: None,
            work: None,
        }
    }
}

impl OperationInfoBuilder
{
    pub fn build(self) -> OperationInfo
    {
        OperationInfo {
            number: self.number.unwrap_or(1),
            work_remaining: self
                .work_remaining
                .expect("`Work` values cannot be missing"),
            work_actual: self.work_actual.expect("`Work` values cannot be missing"),
            work: self.work.expect("`Work` values cannot be missing"),
        }
    }

    pub fn number(mut self, number: NumberOfPeople) -> Self
    {
        self.number = Some(number);
        self
    }

    pub fn work_remaining(mut self, work_remaining: f64) -> Self
    {
        assert!(work_remaining >= 0.0);
        self.work_remaining = Some(Work::from(work_remaining));
        self
    }

    pub fn work_actual(mut self, work_actual: f64) -> Self
    {
        assert!(work_actual >= 0.0);
        self.work_actual = Some(Work::from(work_actual));
        self
    }

    pub fn work(mut self, work: f64) -> Self
    {
        assert!(work >= 0.0);
        self.work = Some(Work::from(work));
        self
    }
}
