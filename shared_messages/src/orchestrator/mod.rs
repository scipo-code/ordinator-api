use actix::Message;
use serde::{Deserialize, Serialize};

use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::WorkOrderNumber;
use crate::models::worker_environment::resources::Id;
use crate::{Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    SetWorkOrderState(WorkOrderNumber, StatusCodes),
    GetWorkOrderStatus(WorkOrderNumber, LevelOfDetail),
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
