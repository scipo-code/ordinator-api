pub mod supervisor_response_resources;
pub mod supervisor_response_scheduling;
pub mod supervisor_response_status;
pub mod supervisor_response_time;
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseResources {}
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseScheduling {}
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseStatus
{
    supervisor_resource: Vec<Id>,
    delegated_work_order_activities: usize,
    objective: SupervisorObjectiveValueResponse,
}

#[derive(Serialize)]
struct SupervisorObjectiveValueResponse {}
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseTime {}
