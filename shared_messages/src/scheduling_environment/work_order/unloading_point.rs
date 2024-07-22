use crate::scheduling_environment::time_environment::period::Period;
use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub period: Option<Period>,
}
