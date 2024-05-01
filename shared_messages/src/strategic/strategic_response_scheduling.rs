use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseScheduling {
    work_orders: Vec<u64>,
    periods: Vec<String>,
}
