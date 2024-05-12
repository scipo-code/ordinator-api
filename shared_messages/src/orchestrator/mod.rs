use actix::Message;
use serde::{Deserialize, Serialize};

use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::WorkOrderNumber;
use crate::models::worker_environment::resources::Id;
use crate::operational::operational_response_status::OperationalResponseStatus;
use crate::strategic::strategic_response_status::StrategicResponseStatus;
use crate::supervisor::supervisor_response_status::SupervisorResponseStatus;
use crate::tactical::tactical_response_status::TacticalResponseStatus;
use crate::{Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    SetWorkOrderState(WorkOrderNumber, StatusCodes),
    GetWorkOrderStatus(WorkOrderNumber, LevelOfDetail),
    GetWorkOrdersState(Asset, LevelOfDetail),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
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

pub struct AgentStatusResponse {
    pub asset: Asset,
    pub agent_status: AgentStatus,
}

pub struct AgentStatus {
    pub strategic_status: StrategicResponseStatus,
    pub tactical_status: TacticalResponseStatus,
    pub supervisor_status: Vec<SupervisorResponseStatus>,
    pub operational_status: Vec<OperationalResponseStatus>,
}

impl AgentStatus {
    pub fn new(
        strategic_status: StrategicResponseStatus,
        tactical_status: TacticalResponseStatus,
        supervisor_status: Vec<SupervisorResponseStatus>,
        operational_status: Vec<OperationalResponseStatus>,
    ) -> Self {
        Self {
            strategic_status,
            tactical_status,
            supervisor_status,
            operational_status,
        }
    }
}
