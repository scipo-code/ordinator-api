use std::collections::{HashMap, HashSet};

use ordinator_scheduling_environment::{
    Asset,
    time_environment::{day::Day, period::Period},
    work_order::{
        WorkOrderNumber,
        operation::Work,
        work_order_analytic::status_codes::{SystemStatusCodes, UserStatusCodes},
        work_order_info::WorkOrderInfo,
    },
    worker_environment::resources::{Id, Resources},
};
use serde::{Deserialize, Serialize};

use crate::{
    operational::responses::operational_response_status::OperationalResponseStatus,
    strategic::responses::strategic_response_status::StrategicResponseStatus,
    supervisor::responses::supervisor_response_status::SupervisorResponseStatus,
    tactical::responses::tactical_response_status::TacticalResponseStatus,
};

// best to simply comment all of this out
// Where should these be found? I think that the
// FIX [ ]
// This should be created with routes and handlers it should all go away
// at somepoint.
#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(WorkOrderNumber),
    GetWorkOrdersState(Asset),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
    // InitializeSystemAgentsFromFile(Asset, ActorSpecifications),
    CreateSupervisorAgent(Asset, u64, Id),
    DeleteSupervisorAgent(Asset, String),
    // CreateOperationalAgent(Asset, Id, f64, OperationalConfiguration),
    DeleteOperationalAgent(Asset, String),
    Export(Asset),
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum OrchestratorResponse {
    AgentStatus(AgentStatusResponse),
    WorkOrderStatus(WorkOrdersStatus),
    RequestStatus(String),
    Periods(Vec<Period>),
    Days(Vec<Day>),
    Export(String),
    Success,
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
    pub supervisor_statai: Vec<SupervisorResponseStatus>,
    pub operational_statai: Vec<OperationalResponseStatus>,
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
            supervisor_statai: supervisor_status,
            operational_statai: operational_status,
        }
    }
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum WorkOrdersStatus {
    Single(WorkOrderResponse),
    SingleSolution(StrategicApiSolution),
    Multiple(HashMap<WorkOrderNumber, WorkOrderResponse>),
}

#[derive(Serialize)]
pub struct WorkOrderResponse {
    earliest_period: Period,
    work_order_info: WorkOrderInfo,
    vendor: bool,
    weight: u64,
    work_order_work_load: HashMap<Resources, Work>,
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
pub struct StrategicApiSolution {
    pub solution: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_period: HashSet<Period>,
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

// TODO [ ]
// Delete this type! These kind of things should always be found in the `conversions`
// crate and not as a stray something in here.
// Should you delete this thing?
// Yes

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
