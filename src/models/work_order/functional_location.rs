
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FunctionalLocation {
    pub string: String
}
