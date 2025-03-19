use ordinator_scheduling_environment::time_environment::period::Period;
use serde::Serialize;

#[derive(Serialize)]
pub struct StrategicResponsePeriods {
    periods: Vec<Period>,
}

impl StrategicResponsePeriods {
    pub fn new(periods: Vec<Period>) -> Self {
        Self { periods }
    }
}
