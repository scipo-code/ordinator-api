use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Priority {
    IntValue(i32),
    StringValue(String),
}

