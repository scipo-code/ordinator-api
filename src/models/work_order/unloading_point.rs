use crate::models::period::PeriodNone;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UnloadingPoint {
    pub string: String,
    pub present: bool,
    pub period: PeriodNone,
}

