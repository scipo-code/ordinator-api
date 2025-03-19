use serde::Deserialize;
use serde::Serialize;

// TODO [x]
// Move to the `configuration`
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeInput {
    pub number_of_strategic_periods: u64,
    pub number_of_tactical_periods: u64,
    pub number_of_days: u64,
    pub number_of_supervisor_periods: u64,
}
