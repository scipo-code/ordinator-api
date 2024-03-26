use crate::agents::tactical_agent::tactical_algorithm::Day;

use self::period::Period;

pub mod period;

pub struct TimeEnvironment {
    pub(super) strategic_periods: Vec<Period>,
    pub(super) tactical_days: Vec<Day>,
}

impl TimeEnvironment {
    pub fn new(strategic_periods: Vec<Period>, tactical_days: Vec<Day>) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_days,
        }
    }

    pub fn strategic_periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }

    pub fn tactical_days(&self) -> &Vec<Day> {
        &self.tactical_days
    }
}
