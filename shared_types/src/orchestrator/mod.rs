use std::collections::HashMap;

use actix::Message;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use crate::operational::operational_response_status::OperationalStatusResponse;
use crate::operational::OperationalConfiguration;
use crate::scheduling_environment::time_environment::day::Day;
use crate::scheduling_environment::time_environment::period::Period;
use crate::scheduling_environment::work_order::status_codes::StatusCodes;
use crate::scheduling_environment::work_order::WorkOrderNumber;
use crate::scheduling_environment::worker_environment::resources::Id;
use crate::strategic::strategic_response_status::{StrategicResponseStatus, WorkOrdersStatus};
use crate::supervisor::supervisor_response_status::SupervisorResponseStatus;
use crate::tactical::tactical_response_status::TacticalResponseStatus;
use crate::{Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest {
    SetWorkOrderState(WorkOrderNumber, StatusCodes),
    GetWorkOrderStatus(WorkOrderNumber, LevelOfDetail),
    GetWorkOrdersState(Asset, LevelOfDetail),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
    CreateSupervisorAgent(Asset, Id),
    DeleteSupervisorAgent(Asset, String),
    CreateOperationalAgent(Asset, Id, OperationalConfiguration),
    DeleteOperationalAgent(Asset, String),
    SetLogLevel(LogLevel),
    SetProfiling(LogLevel),
    Export(Asset),
}

impl Message for OrchestratorRequest {
    type Result = String;
}

#[derive(Serialize)]
pub enum OrchestratorResponse {
    AgentStatus(AgentStatusResponse),
    WorkOrderStatus(WorkOrdersStatus),
    RequestStatus(String),
    Periods(Vec<Period>),
    Days(Vec<Day>),

    Export(String),
}

#[derive(Serialize)]
pub struct AgentStatusResponse {
    pub agent_status: HashMap<Asset, AgentStatus>,
}

impl AgentStatusResponse {
    pub fn new(agent_status: HashMap<Asset, AgentStatus>) -> Self {
        Self { agent_status }
    }
}

#[derive(Serialize)]
pub struct AgentStatus {
    pub strategic_status: StrategicResponseStatus,
    pub tactical_status: TacticalResponseStatus,
    pub supervisor_status: Vec<SupervisorResponseStatus>,
    pub operational_status: Vec<OperationalStatusResponse>,
}

impl AgentStatus {
    pub fn new(
        strategic_status: StrategicResponseStatus,
        tactical_status: TacticalResponseStatus,
        supervisor_status: Vec<SupervisorResponseStatus>,
        operational_status: Vec<OperationalStatusResponse>,
    ) -> Self {
        Self {
            strategic_status,
            tactical_status,
            supervisor_status,
            operational_status,
        }
    }
}
