use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{resources::Id, Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(u32, LevelOfDetail),
    GetWorkOrdersState(Asset, LevelOfDetail),
    GetPeriods,
    GetDays,
    GetAgentStatus,
    CreateSupervisorAgent(Asset, Id),
    DeleteSupervisorAgent(Asset, String),
    CreateOperationalAgent(Asset, Id),
    DeleteOperationalAgent(Asset, String),
    SetLogLevel(LogLevel),
    SetProfiling(LogLevel),
    Export(Asset),
}

impl Message for OrchestratorRequest {
    type Result = String;
}
