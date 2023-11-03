pub mod routes;
pub mod websocket_agent;

use crate::agents::scheduler_agent::scheduler_message::SchedulerRequests;

use serde::{Deserialize};

#[derive(Deserialize)]
#[serde(tag = "message_type")]

enum FrontendMessages {
    Scheduler(SchedulerRequests),
    WorkPlanner,
    Worker,
    Activity,
    WorkCenter,
    WorkOrder
}

