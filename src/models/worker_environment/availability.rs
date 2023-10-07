use chrono::{DateTime, Utc};

pub struct Availability {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
