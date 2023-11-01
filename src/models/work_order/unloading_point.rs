use serde::{Deserialize, Serialize};

use crate::models::period::PeriodNone;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub present: bool,
    pub period: PeriodNone,
}

