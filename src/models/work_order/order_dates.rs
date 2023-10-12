use chrono::{DateTime, Utc, Duration};

pub struct OrderDates {
    pub earliest_allowed_start_date: DateTime<Utc>,
    pub latest_allowed_finish_date: DateTime<Utc>,
    pub basic_start_date: DateTime<Utc>,
    pub basic_finish_date: DateTime<Utc>,
    pub duration: Duration, // Assuming `Day` is another struct or type you've defined
    pub basic_start_scheduled: Option<DateTime<Utc>>,
    pub basic_finish_scheduled: Option<DateTime<Utc>>,
    pub material_expected_date: Option<DateTime<Utc>>,
}