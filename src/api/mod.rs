pub mod routes;
pub mod websocket_agent;

use actix::prelude::*;
use serde::{Deserialize};
use actix_web::{web, App, HttpServer};
use tracing::{Level, info, event};
use std::thread;

use crate::api::routes::ws_index;
use crate::agents::scheduler_agent::scheduler_message::SchedulerRequests;
use crate::agents::scheduler_agent::SchedulerAgent;

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


trait WebSocketAgentTrait {
    fn new() -> Self;
}
