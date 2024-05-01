use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicStatusMessage {
    General,
    Period(String),
}

impl StrategicStatusMessage {
    pub fn new_period(period: String) -> Self {
        Self::Period(period)
    }
}

impl Display for StrategicStatusMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategicStatusMessage::General => write!(f, "general"),
            StrategicStatusMessage::Period(period) => write!(f, "period: {}", period),
        }
    }
}
