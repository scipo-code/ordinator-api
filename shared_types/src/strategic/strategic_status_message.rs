use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StatusRequestMessage {
    General,
    Period(String),
}

impl StatusRequestMessage {
    pub fn new_period(period: String) -> Self {
        Self::Period(period)
    }
}

impl Display for StatusRequestMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusRequestMessage::General => write!(f, "general"),
            StatusRequestMessage::Period(period) => write!(f, "period: {}", period),
        }
    }
}
