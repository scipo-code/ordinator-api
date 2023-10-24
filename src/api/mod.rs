pub mod routes;
pub mod websocket_agent;

use crate::agents::scheduler_agent::scheduler_message::SchedulerMessages;

use serde::{Deserialize};

#[derive(Deserialize)]
#[serde(tag = "message_type")]

enum FrontendMessages {
    Scheduler(SchedulerMessages),
    WorkPlanner,
    Worker,
    Activity,
    WorkCenter,
    WorkOrder
}

