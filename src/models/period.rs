use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum PeriodNone {
    Period(Period),
    None,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Period {
    id: u32,
    period_string: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl Period {
    pub fn new(id: u32, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Period {
        let period_string = "{(year(start_date)}-W{week(start_date)}-{week(end_date)-1 == 0 ? 52 : week(end_date)-1}";
        Period { id: id, period_string: period_string.to_string(), start_date: start_date, end_date: end_date}
    }
}

impl Period {
    pub fn get_period(&self) -> String {
        self.period_string.clone()
    }
}