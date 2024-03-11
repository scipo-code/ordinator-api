use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{resources::Id, LevelOfDetail};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(u32, LevelOfDetail),
    GetWorkOrdersState(LevelOfDetail),
    GetPeriods,
    GetAgentStatus,
    CreateSupervisorAgent(Id),
    DeleteSupervisorAgent(String),
    CreateOperationalAgent(Id),
    DeleteOperationalAgent(String),
}

impl Message for OrchestratorRequest {
    type Result = String;
}
