use crate::models::time_environment::period::Period;
use clap::{Args};
use serde::{Deserialize, Serialize};

#[derive(Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint {
    pub string: String,

    pub period: Option<Period>,
}

impl Default for UnloadingPoint {
    fn default() -> Self {
        UnloadingPoint {
            string: String::from(""),
            period: None,
        }
    }
}
