use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Berlin;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Availability {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
