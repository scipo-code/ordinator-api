use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(u32),
    GetPeriods,
}

impl Message for OrchestratorRequest {
    type Result = String;
}
