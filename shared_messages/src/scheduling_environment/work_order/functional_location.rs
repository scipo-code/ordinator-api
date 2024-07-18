use crate::Asset;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionalLocation {
    pub string: String,
    pub asset: Asset,
}

impl Default for FunctionalLocation {
    fn default() -> Self {
        FunctionalLocation {
            string: "Unknown".to_string(),
            asset: Asset::Unknown,
        }
    }
}
