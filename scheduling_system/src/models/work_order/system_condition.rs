use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub enum SystemCondition {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    Unknown
}

impl SystemCondition {
    pub fn new() -> Self {
        SystemCondition::Unknown
    }
}