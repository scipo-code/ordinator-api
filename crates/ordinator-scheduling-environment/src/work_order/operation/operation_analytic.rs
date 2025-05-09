use serde::Deserialize;
use serde::Serialize;

use super::Work;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationAnalytic
{
    pub preparation_time: Work,
    // FIX
    // This is wrong, this field should be given completely by the
    // numbers and `work_remaining` you should integrate this into
    // the builder.
    pub duration: Work,
}

pub struct OperationAnalyticBuilder
{
    preparation_time: Option<Work>,
    duration: Option<Work>,
}

impl OperationAnalyticBuilder
{
    pub fn build(self) -> OperationAnalytic
    {
        OperationAnalytic {
            preparation_time: self.preparation_time.unwrap_or_default(),
            duration: self.duration.unwrap_or_default(),
        }
    }

    pub fn preparation_time(mut self, preparation_time: f64) -> Self
    {
        self.preparation_time = Some(Work::from(preparation_time));
        self
    }

    pub fn duration(mut self, duration: f64) -> Self
    {
        self.duration = Some(Work::from(duration));
        self
    }
}

impl OperationAnalytic
{
    pub fn builder() -> OperationAnalyticBuilder
    {
        OperationAnalyticBuilder {
            preparation_time: None,
            duration: None,
        }
    }
}
