use chrono::TimeZone;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Deserializer;

#[derive(Deserialize)]
pub struct Availability {
    #[serde(
        deserialize_with = "crate::models::worker_environment::availability::deserialize_datetime"
    )]
    start_date: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_datetime")]
    end_date: DateTime<Utc>,
}

// Custom deserialization function
pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
        .map_err(serde::de::Error::custom)
}
