pub mod routes;
pub mod websocket_agent;

use serde::Deserialize;

use crate::agents::scheduler_agent::scheduler_message::SchedulerRequests;

#[derive(Deserialize, Debug)]
#[serde(tag = "message_type")]
enum FrontendMessages {
    Scheduler(SchedulerRequests),
    WorkPlanner,
    Worker,
    Activity,
    WorkCenter,
    WorkOrder,
}

trait WebSocketAgentTrait {
    fn new() -> Self;
}
