use chrono::{DateTime, Utc};

#[warn(dead_code)]
pub struct Availability {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
