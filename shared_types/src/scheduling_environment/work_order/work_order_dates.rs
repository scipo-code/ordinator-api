use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::scheduling_environment::time_environment::period::Period;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderDates {
    pub earliest_allowed_start_date: DateTime<Utc>,
    pub latest_allowed_finish_date: DateTime<Utc>,
    pub earliest_allowed_start_period: Period,
    pub latest_allowed_finish_period: Period,
    pub basic_start_date: DateTime<Utc>,
    pub basic_finish_date: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub duration: Duration,
    pub basic_start_scheduled: Option<DateTime<Utc>>,
    pub basic_finish_scheduled: Option<DateTime<Utc>>,
    pub material_expected_date: Option<DateTime<Utc>>,
}

impl WorkOrderDates {
    pub fn new(
        earliest_allowed_start_date: DateTime<Utc>,
        latest_allowed_finish_date: DateTime<Utc>,
        earliest_allowed_start_period: Period,
        latest_allowed_finish_period: Period,
        basic_start_date: DateTime<Utc>,
        basic_finish_date: DateTime<Utc>,
        duration: Duration,
        basic_start_scheduled: Option<DateTime<Utc>>,
        basic_finish_scheduled: Option<DateTime<Utc>>,
        material_expected_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            earliest_allowed_start_date,
            latest_allowed_finish_date,
            earliest_allowed_start_period,
            latest_allowed_finish_period,
            basic_start_date,
            basic_finish_date,
            duration,
            basic_start_scheduled,
            basic_finish_scheduled,
            material_expected_date,
        }
    }

    pub fn new_test() -> Self {
        use chrono::TimeZone;
        Self {
            earliest_allowed_start_date: Utc.with_ymd_and_hms(2023, 10, 20, 0, 0, 0).unwrap(),
            latest_allowed_finish_date: Utc.with_ymd_and_hms(2023, 12, 20, 0, 0, 0).unwrap(),
            earliest_allowed_start_period: Period::default(),
            latest_allowed_finish_period: Period::default(),
            basic_start_date: Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap(),
            basic_finish_date: Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap(),
            duration: Duration::seconds(0),
            basic_start_scheduled: None,
            basic_finish_scheduled: None,
            material_expected_date: None,
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
