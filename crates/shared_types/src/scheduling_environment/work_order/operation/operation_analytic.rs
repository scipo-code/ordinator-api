use serde::Deserialize;
use serde::Serialize;

use super::Work;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationAnalytic {
    pub preparation_time: Work,
    pub duration: Option<Work>,
}

impl OperationAnalytic {
    pub fn new(preparation_time: Work, duration: Option<Work>) -> Self {
        OperationAnalytic {
            preparation_time,
            duration,
        }
    }
}
