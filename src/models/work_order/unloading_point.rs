use serde::{Deserialize, Serialize};
use crate::models::period::Period;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub present: bool,
    pub period: Option<Period>,
}

