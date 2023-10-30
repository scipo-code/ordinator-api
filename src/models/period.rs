use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub enum PeriodNone {
    Period(Period),
    None,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Period {
    id: u32,
    pub period_string: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl Period {
    pub fn new(id: u32, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Period {
        dbg!(start_date);
        dbg!(start_date.iso_week());
        dbg!(start_date.iso_week().week());

        dbg!(end_date);
        dbg!(end_date.iso_week());
        dbg!(end_date.iso_week().week());

        let period_string = format!("{}-W{}-{}",
            start_date.year(),
            start_date.iso_week().week(),
            if end_date.iso_week().week() == 1 { 52 } else { end_date.iso_week().week() });
        Period { id: id, period_string: period_string, start_date: start_date, end_date: end_date}
    }
}

impl Period {
    pub fn get_string(&self) -> String {
        self.period_string.clone()
    }

    pub fn get_start_date(&self) -> DateTime<Utc> {
        self.start_date
    }

    pub fn get_end_date(&self) -> DateTime<Utc> {
        self.end_date
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Period: {}", self.period_string)
    }
}