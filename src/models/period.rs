use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum PeriodNone {
    Period(Period),
    None,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[derive(Debug)]
#[derive(Clone)]
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
        write!(f, 
            "Period: {}, \n
            Start Date: {}, \n
            End Date: {}", 
            self.period_string, 
            self.start_date, 
            self.end_date)

    }
}