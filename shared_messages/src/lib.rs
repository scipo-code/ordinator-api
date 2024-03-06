pub mod agent_error;
pub mod orchestrator;
pub mod resources;
pub mod strategic;
pub mod tactical;
use actix::prelude::*;
use orchestrator::OrchestratorRequest;
use serde::{Deserialize, Serialize};

use crate::strategic::StrategicRequest;
use crate::tactical::TacticalRequest;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "message_type")]
pub enum SystemMessages {
    Orchestrator(OrchestratorRequest),
    Strategic(StrategicRequest),
    Tactical(TacticalRequest),
    Supervisor,
    Operational,
    Sap,
}

impl Message for SystemMessages {
    type Result = ();
}

pub struct StopMessage {}

impl Message for StopMessage {
    type Result = ();
}

pub struct StatusMessage {}

impl Message for StatusMessage {
    type Result = String;
}
