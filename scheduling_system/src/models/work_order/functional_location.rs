use serde::{Deserialize, Serialize};
use shared_messages::Asset;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionalLocation {
    pub string: String,
    pub asset: Asset,
}

impl FunctionalLocation {
    #[cfg(test)]
    pub fn new_default() -> Self {
        FunctionalLocation {
            string: "testing-stub-for-functional-location".to_string(),
            asset: Asset::DF,
        }
    }
}
