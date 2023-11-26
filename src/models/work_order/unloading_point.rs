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

impl UnloadingPoint {

    #[cfg(test)]
    pub fn new_default() -> Self {
        UnloadingPoint {
            string: String::from(""),
            present: false,
            period: None,
        }
    }
}