use std::collections::HashMap;
use std::collections::HashSet;

use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::SystemStatusCodes;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::UserStatusCodes;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfo;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;

// best to simply comment all of this out
// Where should these be found? I think that the
// FIX [ ]
// This should be created with routes and handlers it should all go away
// at somepoint.
// I guess that this should be inside of the Orchestrator instead. What
// other approach should we choose here? I think that creating the
// This should lie inside of the Orchestrator. I do not see a way
// around it.

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum OrchestratorResponse
{
    WorkOrderStatus(WorkOrdersStatus),
    RequestStatus(String),
    Periods(Vec<Period>),
    Days(Vec<Day>),
    Export(String),
    Success,
    Todo,
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum WorkOrdersStatus
{
    Single(WorkOrderResponse),
    SingleSolution(StrategicApiSolution),
    Multiple(HashMap<WorkOrderNumber, WorkOrderResponse>),
}

#[derive(Serialize)]
pub struct WorkOrderResponse
{
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
pub struct ApiSolution
{
    pub strategic: String,   //ApiStrategic,
    pub tactical: String,    //ApiTactical,
    pub supervisor: String,  //HashMap<Id, ApiSupervisor>,
    pub operational: String, //HashMap<Id, ApiOperational>,
}

#[derive(Serialize)]
pub struct StrategicApiSolution
{
    pub solution: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_period: HashSet<Period>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiStrategic
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiTactical
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiSupervisor
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiOperational
{
    solution_data: String,
}

// TODO [ ]
// Delete this type! These kind of things should always be found in the
// `conversions` crate and not as a stray something in here.
// Should you delete this thing?
// Yes

#[derive(Serialize)]
pub struct OptimizedWorkOrderResponse
{
    scheduled_period: Period,
    locked_in_period: Option<Period>,
    excluded_periods: HashSet<Period>,
    latest_period: Period,
}

impl OptimizedWorkOrderResponse
{
    pub fn new(
        scheduled_period: Period,
        locked_in_period: Option<Period>,
        excluded_periods: HashSet<Period>,
        latest_period: Period,
    ) -> Self
    {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_periods,
            latest_period,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrchestratorMessage<T>
{
    pub message_from_orchestrator: T,
}

impl<T> OrchestratorMessage<T>
{
    pub fn new(message_from_orchestrator: T) -> Self
    {
        Self {
            message_from_orchestrator,
        }
    }
}
