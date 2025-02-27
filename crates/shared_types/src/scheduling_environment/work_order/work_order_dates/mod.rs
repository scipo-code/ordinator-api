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
    pub latest_allowed_finish_period: Period,
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

pub struct WorkOrderDatesBuilder {
    earliest_allowed_start_date: NaiveDate,
    latest_allowed_finish_date: NaiveDate,
    latest_allowed_finish_period: Period,
    basic_start_date: NaiveDate,
    basic_finish_date: NaiveDate,
    duration: Duration,
    basic_start_scheduled: Option<DateTime<Utc>>,
    basic_finish_scheduled: Option<DateTime<Utc>>,
    material_expected_date: Option<DateTime<Utc>>,
}

impl WorkOrderDates {
    pub fn builder() -> WorkOrderDatesBuilder {
        WorkOrderDatesBuilder {
            earliest_allowed_start_date: todo!(),
            latest_allowed_finish_date: todo!(),
            latest_allowed_finish_period: todo!(),
            basic_start_date: todo!(),
            basic_finish_date: todo!(),
            duration: todo!(),
            basic_start_scheduled: todo!(),
            basic_finish_scheduled: todo!(),
            material_expected_date: todo!(),
        }
    }
}

impl WorkOrderDatesBuilder {
    pub fn build(self) -> WorkOrderDates {
        WorkOrderDates {
            earliest_allowed_start_date: todo!(),
            latest_allowed_finish_date: todo!(),
            latest_allowed_finish_period: todo!(),
            basic_start_date: todo!(),
            basic_finish_date: todo!(),
            duration: todo!(),
            basic_start_scheduled: todo!(),
            basic_finish_scheduled: todo!(),
            material_expected_date: todo!(),
        }
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
