use std::collections::{HashMap, HashSet};

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::operational::operational_response_status::OperationalStatusResponse;
use crate::operational::OperationalConfiguration;
use crate::scheduling_environment::time_environment::day::Day;
use crate::scheduling_environment::time_environment::period::Period;
use crate::scheduling_environment::work_order::status_codes::{SystemStatusCodes, UserStatusCodes};
use crate::scheduling_environment::work_order::{WorkOrder, WorkOrderInfo, WorkOrderNumber};
use crate::scheduling_environment::worker_environment::resources::Id;
use crate::strategic::strategic_response_status::StrategicResponseStatus;
use crate::supervisor::supervisor_response_status::SupervisorResponseStatus;
use crate::tactical::tactical_response_status::TacticalResponseStatus;
use crate::{Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest {
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

#[derive(Serialize)]
pub enum WorkOrdersStatus {
    Single(WorkOrderResponse),
    Multiple(HashMap<WorkOrderNumber, WorkOrderResponse>),
}

#[derive(Serialize)]
pub struct WorkOrderResponse {
    earliest_period: Period,
    work_order_info: WorkOrderInfo,
    vendor: bool,
    weight: u64,
    system_status_codes: SystemStatusCodes,
    user_status_codes: UserStatusCodes,
    api_solution: ApiSolution,
}

#[derive(Serialize)]
pub struct ApiSolution {
    pub strategic: String,   //ApiStrategic,
    pub tactical: String,    //ApiTactical,
    pub supervisor: String,  //HashMap<Id, ApiSupervisor>,
    pub operational: String, //HashMap<Id, ApiOperational>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiStrategic {
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiTactical {
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiSupervisor {
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiOperational {
    solution_data: String,
}

impl WorkOrderResponse {
    pub fn new(work_order: &WorkOrder, api_solution: ApiSolution) -> Self {
        let earliest_period = work_order
            .work_order_dates
            .earliest_allowed_start_period
            .clone();
        let work_order_info = work_order.work_order_info.clone();
        let vendor = work_order.work_order_analytic.vendor;
        let weight = work_order.work_order_analytic.work_order_weight;
        let system_status_codes = work_order.work_order_analytic.system_status_codes.clone();
        let user_status_codes = work_order.work_order_analytic.user_status_codes.clone();

        Self {
            earliest_period,
            work_order_info,
            vendor,
            weight,
            system_status_codes,
            user_status_codes,
            api_solution,
        }
    }
}

#[derive(Serialize)]
pub struct OptimizedWorkOrderResponse {
    scheduled_period: Period,
    locked_in_period: Option<Period>,
    excluded_periods: HashSet<Period>,
    latest_period: Period,
}

impl OptimizedWorkOrderResponse {
    pub fn new(
        scheduled_period: Period,
        locked_in_period: Option<Period>,
        excluded_periods: HashSet<Period>,
        latest_period: Period,
    ) -> Self {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_periods,
            latest_period,
        }
    }
}

impl Message for OrchestratorRequest {
    type Result = String;
}

#[derive(Clone, Debug)]
pub struct OrchestratorMessage<T> {
    pub message_from_orchestrator: T,
}

impl<T> OrchestratorMessage<T> {
    pub fn new(message_from_orchestrator: T) -> Self {
        Self {
            message_from_orchestrator,
        }
    }
}

impl<T> Message for OrchestratorMessage<T> {
    type Result = ();
}
