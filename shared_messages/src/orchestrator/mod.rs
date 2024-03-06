use actix::Message;
use serde::{Deserialize, Serialize};

use crate::resources::{Id, Resources};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(u32),
    GetPeriods,
    GetAgentStatus,
    CreateSupervisorAgent(Id, Resources),
    DeleteSupervisorAgent(Id),
    CreateOperationalAgent(Id, Vec<Resources>),
    DeleteOperationalAgent(Id),
}

impl Message for OrchestratorRequest {
    type Result = String;
}
