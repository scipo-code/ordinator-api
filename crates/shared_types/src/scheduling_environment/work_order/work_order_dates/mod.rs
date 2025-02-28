pub mod unloading_point;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::scheduling_environment::time_environment::period::Period;

use super::work_order_analytic::status_codes::MaterialStatus;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderDates {
    pub earliest_allowed_start_date: NaiveDate,
    pub latest_allowed_finish_date: NaiveDate,
    // TODO [ ]
    // This should be a function. It can be uniquely
    // derived from the other fields.
    pub basic_start_date: NaiveDate,
    pub basic_finish_date: NaiveDate,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub duration: Duration,
    pub basic_start_scheduled: Option<DateTime<Utc>>,
    pub basic_finish_scheduled: Option<DateTime<Utc>>,
    // TODO [ ]
    // This should be a function. It can be uniquely
    // derived from the other fields.
    pub material_expected_date: Option<DateTime<Utc>>,
}

// FIX
// Find the latest allowed finish period and replace it with a function.
pub struct WorkOrderDatesBuilder {
    earliest_allowed_start_date: Option<NaiveDate>,
    latest_allowed_finish_date: Option<NaiveDate>,
    basic_start_date: Option<NaiveDate>,
    basic_finish_date: Option<NaiveDate>,
    duration: Option<Duration>,
    basic_start_scheduled: Option<DateTime<Utc>>,
    basic_finish_scheduled: Option<DateTime<Utc>>,
    material_expected_date: Option<DateTime<Utc>>,
}

impl WorkOrderDates {
    pub fn builder() -> WorkOrderDatesBuilder {
        WorkOrderDatesBuilder {
            earliest_allowed_start_date: None,
            latest_allowed_finish_date: None,
            basic_start_date: None,
            basic_finish_date: None,
            duration: None,
            basic_start_scheduled: None,
            basic_finish_scheduled: None,
            material_expected_date: None,
        }
    }
}

impl WorkOrderDatesBuilder {
    pub fn build(self) -> WorkOrderDates {
        WorkOrderDates {
            earliest_allowed_start_date: self.earliest_allowed_start_date.expect("This is a mandatory field if you are having runtime issue make the builder strong typed") ,
            latest_allowed_finish_date: self.latest_allowed_finish_date.expect("This is a mandatory field if you are having runtime issue make the builder strong typed") ,
            basic_start_date: self.basic_start_date.expect("This is a mandatory field if you are having runtime issue make the builder strong typed") ,
            basic_finish_date: self.basic_finish_date.expect("This is a mandatory field if you are having runtime issue make the builder strong typed") ,
            duration: self.duration.expect("This is a mandatory field if you are having runtime issue make the builder strong typed") ,
            basic_start_scheduled: self.basic_start_scheduled ,
            basic_finish_scheduled: self.basic_finish_scheduled ,
            material_expected_date: self.material_expected_date ,
        }
    }
    pub fn earliest_allowed_start_date(mut self, earliest_allowed_start_date: NaiveDate) -> Self {
        self.earliest_allowed_start_date = Some(earliest_allowed_start_date);
        self
    }
    pub fn latest_allowed_finish_date(mut self, latest_allowed_finish_date: NaiveDate) -> Self {
        self.latest_allowed_finish_date = Some(latest_allowed_finish_date);
        self
    }
    pub fn basic_start_date(mut self, basic_start_date: NaiveDate) -> Self {
        self.basic_start_date = Some(basic_start_date);
        self
    }
    pub fn basic_finish_date(mut self, basic_finish_date: NaiveDate) -> Self {
        self.basic_finish_date = Some(basic_finish_date);
        self
    }
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    pub fn basic_start_scheduled(mut self, basic_start_scheduled: DateTime<Utc>) -> Self {
        self.basic_start_scheduled = Some(basic_start_scheduled);
        self
    }
    pub fn basic_finish_scheduled(mut self, basic_finish_scheduled: DateTime<Utc>) -> Self {
        self.basic_finish_scheduled = Some(basic_finish_scheduled);
        self
    }
    pub fn material_expected_date(mut self, material_expected_date: DateTime<Utc>) -> Self {
        self.material_expected_date = Some(material_expected_date);
        self
    }
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let secs = duration.num_seconds();
    serializer.serialize_i64(secs)
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = i64::deserialize(deserializer)?;
    Ok(Duration::seconds(secs))
}
