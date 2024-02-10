use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusRequest {
    GetWorkOrderStatus(u32),
    GetPeriods,
}

impl Message for StatusRequest {
    type Result = ();
}
