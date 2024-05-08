use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Berlin;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Availability {
    #[serde(deserialize_with = "deserialize_datetime")]
    #[allow(dead_code)]
    start_date: DateTime<Utc>,
    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_datetime")]
    end_date: DateTime<Utc>,
}

// Custom deserialization function
pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let native_date_time =
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").expect("Wrong date inputed");

    let danish_date_time = Berlin.from_local_datetime(&native_date_time).unwrap();
    Ok(danish_date_time.with_timezone(&Utc))
}
