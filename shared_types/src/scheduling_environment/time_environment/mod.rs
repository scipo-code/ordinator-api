use serde::{Deserialize, Serialize};

use self::day::Day;
use self::period::Period;

pub mod day;
pub mod period;

// WARN: Make the fields private. It does not make sense to change these individually.
#[derive(Serialize, Deserialize, Debug)]
pub struct TimeEnvironment {
    strategic_periods: Vec<Period>,
    tactical_periods: Vec<Period>,
    tactical_days: Vec<Day>,
}

impl TimeEnvironment {
    pub fn new(
        strategic_periods: Vec<Period>,
        tactical_periods: Vec<Period>,
        tactical_days: Vec<Day>,
    ) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_periods,
            tactical_days,
        }
    }

    pub fn strategic_periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }
    pub fn strategic_periods_mut(&mut self) -> &mut Vec<Period> {
        &mut self.strategic_periods
    }

    pub fn tactical_periods(&self) -> &Vec<Period> {
        &self.tactical_periods
    }

    pub fn tactical_days(&self) -> &Vec<Day> {
        &self.tactical_days
    }
}
