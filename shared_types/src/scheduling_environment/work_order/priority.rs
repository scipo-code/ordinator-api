use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Priority {
    IntValue(u64),
    StringValue(String),
}

impl Priority {
    pub fn get_priority_string(&self) -> String {
        match self {
            Priority::IntValue(priority) => priority.to_string(),
            Priority::StringValue(priority) => priority.to_string(),
        }
    }
}

impl Priority {
    pub fn new_int(priority: u64) -> Self {
        Self::IntValue(priority)
    }
}
