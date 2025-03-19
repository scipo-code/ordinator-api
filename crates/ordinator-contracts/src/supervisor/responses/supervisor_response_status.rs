use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

#[derive(Serialize)]
pub struct SupervisorResponseStatus {
    supervisor_resource: Vec<Id>,
    delegated_work_order_activities: usize,
    objective: SupervisorObjectiveValueResponse,
}

#[derive(Serialize)]
struct SupervisorObjectiveValueResponse {}
