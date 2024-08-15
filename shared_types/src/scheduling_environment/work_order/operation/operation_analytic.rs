use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationAnalytic {
    pub preparation_time: f64,
    pub duration: f64,
}

impl OperationAnalytic {
    pub fn new(preparation_time: f64, duration: f64) -> Self {
        OperationAnalytic {
            preparation_time,
            duration,
        }
    }
    pub fn duration(&self) -> f64 {
        self.duration
    }
}
