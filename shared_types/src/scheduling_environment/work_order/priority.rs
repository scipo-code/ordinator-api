use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Priority {
    IntValue(u32),
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
    pub fn new_int(priority: u32) -> Self {
        Self::IntValue(priority)
    }
}
