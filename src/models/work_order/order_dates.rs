use chrono::{DateTime, Utc, Duration};

pub struct OrderDates {
    earliest_allowed_start_date: DateTime<Utc>,
    latest_allowed_finish_date: DateTime<Utc>,
    basic_start: DateTime<Utc>,
    basic_finish: DateTime<Utc>,
    duration: Duration, // Assuming `Day` is another struct or type you've defined
    basic_start_scheduled: DateTime<Utc>,
    basic_finish_scheduled: DateTime<Utc>,
    material_expected_date: Option<DateTime<Utc>>,
}