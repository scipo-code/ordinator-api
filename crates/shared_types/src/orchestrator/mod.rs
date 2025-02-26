use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::agents::operational::responses::operational_response_status::OperationalResponseStatus;
use crate::agents::operational::OperationalConfiguration;
use crate::agents::strategic::responses::strategic_response_status::StrategicResponseStatus;
use crate::agents::supervisor::responses::supervisor_response_status::SupervisorResponseStatus;
use crate::agents::tactical::responses::tactical_response_status::TacticalResponseStatus;
use crate::scheduling_environment::time_environment::day::Day;
use crate::scheduling_environment::time_environment::period::Period;
use crate::scheduling_environment::work_order::operation::Work;
use crate::scheduling_environment::work_order::work_order_analytic::status_codes::{
    SystemStatusCodes, UserStatusCodes,
};
use crate::scheduling_environment::work_order::WorkOrderConfigurations;
use crate::scheduling_environment::work_order::{
    work_order_info::WorkOrderInfo, WorkOrder, WorkOrderNumber,
};
use crate::scheduling_environment::worker_environment::resources::{Id, Resources};
use crate::{ActorSpecifications, Asset, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(WorkOrderNumber, LevelOfDetail),
    GetWorkOrdersState(Asset, LevelOfDetail),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
    InitializeSystemAgentsFromFile(Asset, ActorSpecifications),
    CreateSupervisorAgent(Asset, u64, Id),
    DeleteSupervisorAgent(Asset, String),
    CreateOperationalAgent(Asset, Id, f64, OperationalConfiguration),
    DeleteOperationalAgent(Asset, String),
    SetLogLevel(LogLevel),
    SetProfiling(LogLevel),
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

/// Should you delete this thing?
impl WorkOrderResponse {
    pub fn new(
        work_order: &WorkOrder,
        api_solution: ApiSolution,
        work_order_configurations: &WorkOrderConfigurations,
    ) -> Self {
        // WARN
        // Crucial lesson here. Derived needed information with functions allows you to
        // expose weak structures in data flow. Below we see that introducing a function
        // makes the code require an argument.
        // QUESTION
        // Do you even need `Periods` can they not always be derived instead? I think that
        // they can.
        let earliest_period = work_order.earliest_allowed_start_period(periods).clone();

        let work_order_info = work_order.work_order_info.clone();
        let work_order_work_load = work_order.work_order_load();
        let vendor = work_order.vendor();
        // This is a good sign. You should be able to provide the work_order_configurations for this
        // and the `MessageHandler` trait has to be updated.
        let weight = work_order.work_order_value(work_order_configurations);
        let system_status_codes = work_order.work_order_analytic.system_status_codes.clone();
        let user_status_codes = work_order.work_order_analytic.user_status_codes.clone();

        Self {
            earliest_period,
            work_order_info,
            vendor,
            weight,
            work_order_work_load,
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
