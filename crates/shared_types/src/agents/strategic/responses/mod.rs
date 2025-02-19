pub mod strategic_response_periods;
pub mod strategic_response_resources;
pub mod strategic_response_scheduling;
pub mod strategic_response_status;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StrategicObjectiveValueResponse {
    field_one: String,
}
