use serde::{Deserialize, Serialize};

use self::day::Day;
use self::period::Period;

pub mod day;
pub mod period;

// WARN: Make the fields private. It does not make sense to change these individually.
// FIX
// All Periods here refer to the same thing. You should use references
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TimeEnvironment {
    pub strategic_periods: Vec<Period>,
    pub tactical_periods: Vec<Period>,
    pub tactical_days: Vec<Day>,
    pub supervisor_periods: Vec<Period>,
}

impl TimeEnvironment {
    pub fn new(
        strategic_periods: Vec<Period>,
        tactical_periods: Vec<Period>,
        tactical_days: Vec<Day>,
        supervisor_periods: Vec<Period>,
    ) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_periods,
            tactical_days,
            supervisor_periods,
        }
    }
}
