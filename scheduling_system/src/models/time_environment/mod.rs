use chrono::NaiveDate;

use self::period::Period;

pub mod period;

pub struct TimeEnvironment {
    pub(super) strategic_periods: Vec<Period>,
    pub(super) tactical_days: Vec<NaiveDate>,
}

impl TimeEnvironment {
    pub fn new(strategic_periods: Vec<Period>, tactical_days: Vec<NaiveDate>) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_days,
        }
    }

    pub fn get_strategic_periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }

    pub fn get_tactical_days(&self) -> &Vec<NaiveDate> {
        &self.tactical_days
    }
}
