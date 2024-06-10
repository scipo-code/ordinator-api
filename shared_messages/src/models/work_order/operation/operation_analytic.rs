use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationAnalytic {
    pub preparation_time: f64,
    pub duration: u32,
}

impl OperationAnalytic {
    pub fn new(preparation_time: f64, duration: u32) -> Self {
        OperationAnalytic {
            preparation_time,
            duration,
        }
    }
    pub fn duration(&self) -> u32 {
        self.duration
    }
}
