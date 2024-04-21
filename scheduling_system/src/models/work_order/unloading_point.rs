use crate::models::time_environment::period::Period;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub period: Option<Period>,
}

impl UnloadingPoint {
    #[cfg(test)]
    pub fn new_default() -> Self {
        UnloadingPoint {
            string: String::from(""),
            period: None,
        }
    }
}
