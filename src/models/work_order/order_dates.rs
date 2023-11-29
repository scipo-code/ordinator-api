use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct OrderDates {
    pub earliest_allowed_start_date: DateTime<Utc>,
    pub latest_allowed_finish_date: DateTime<Utc>,
    pub basic_start_date: DateTime<Utc>,
    pub basic_finish_date: DateTime<Utc>,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration, // Assuming `Day` is another struct or type you've defined
    pub basic_start_scheduled: Option<DateTime<Utc>>,
    pub basic_finish_scheduled: Option<DateTime<Utc>>,
    pub material_expected_date: Option<DateTime<Utc>>,
}

impl OrderDates {
    #[cfg(test)]
    pub fn new_default() -> Self {
        Self {
            earliest_allowed_start_date: Utc::now(),
            latest_allowed_finish_date: Utc::now(),
            basic_start_date: Utc::now(),
            basic_finish_date: Utc::now(),
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