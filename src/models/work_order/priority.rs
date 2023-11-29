use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Priority {
    IntValue(u32),
    StringValue(String),
}

impl Priority {
    #[cfg(test)]
    pub fn new_int(priority: u32) -> Self {
        Self::IntValue(priority)
    }
}