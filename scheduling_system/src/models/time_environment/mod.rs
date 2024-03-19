use chrono::{DateTime, Utc};

use self::period::Period;

pub mod period;

pub struct TimeEnvironment {
    pub(super) strategic_periods: Vec<Period>,
    pub(super) tactical_days: Vec<DateTime<Utc>>,
}

impl TimeEnvironment {
    pub fn new(strategic_periods: Vec<Period>, tactical_days: Vec<DateTime<Utc>>) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_days,
        }
    }

    pub fn get_strategic_periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }

    pub fn get_tactical_days(&self) -> &Vec<DateTime<Utc>> {
        &self.tactical_days
    }
}
