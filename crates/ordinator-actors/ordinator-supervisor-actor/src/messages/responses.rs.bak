use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseResources {}

#[derive(Serialize)]
pub struct SupervisorResponseScheduling {}

#[derive(Serialize)]
pub struct SupervisorResponseStatus
{
    pub supervisor_resource: Vec<Id>,
    pub delegated_work_order_activities: usize,
    pub objective: u64,
}

#[derive(Serialize)]
pub struct SupervisorResponseTime {}
