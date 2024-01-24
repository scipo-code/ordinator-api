use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct FunctionalLocation {
    pub string: String
}

impl FunctionalLocation {
    #[cfg(test)]
    pub fn new_default() -> Self {
        FunctionalLocation {
            string: "testing-stub-for-functional-location".to_string()
        }
    }
}