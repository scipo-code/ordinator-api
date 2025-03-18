pub mod operational;
pub mod orchestrator;
pub mod strategic;
pub mod supervisor;
pub mod tactical;

use operational::{OperationalRequest, OperationalResponse};
use orchestrator::{OrchestratorRequest, OrchestratorResponse};
use serde::{Deserialize, Serialize};
use strategic::{StrategicRequest, StrategicResponse};
use supervisor::{SupervisorRequest, SupervisorResponse};
use tactical::{TacticalRequest, TacticalResponse};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "message_type")]
pub enum SystemMessages {
    Orchestrator(OrchestratorRequest),
    Strategic(StrategicRequest),
    Tactical(TacticalRequest),
    Supervisor(SupervisorRequest),
    Operational(OperationalRequest),
    Sap,
}

#[derive(Serialize)]
pub enum SystemResponses {
    Orchestrator(OrchestratorResponse),
    Strategic(StrategicResponse),
    Tactical(TacticalResponse),
    Supervisor(SupervisorResponse),
    Operational(OperationalResponse),
    Export,
    Sap,
}
