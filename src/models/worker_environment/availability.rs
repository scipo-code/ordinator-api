use chrono::{DateTime, Utc};

#[allow(dead_code)]
pub struct Availability {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
