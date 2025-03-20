pub mod operational;
pub mod orchestrator;
pub mod strategic;
pub mod supervisor;
pub mod tactical;

use operational::OperationalRequest;
use operational::OperationalResponse;
use orchestrator::OrchestratorRequest;
use orchestrator::OrchestratorResponse;
use serde::Deserialize;
use serde::Serialize;
use strategic::StrategicRequest;
use strategic::StrategicResponse;
use supervisor::SupervisorRequest;
use supervisor::SupervisorResponse;
use tactical::TacticalRequest;
use tactical::TacticalResponse;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "message_type")]
pub enum SystemMessages
{
    Orchestrator(OrchestratorRequest),
    Strategic(StrategicRequest),
    Tactical(TacticalRequest),
    Supervisor(SupervisorRequest),
    Operational(OperationalRequest),
    Sap,
}

#[derive(Serialize)]
pub enum SystemResponses
{
    Orchestrator(OrchestratorResponse),
    Strategic(StrategicResponse),
    Tactical(TacticalResponse),
    Supervisor(SupervisorResponse),
    Operational(OperationalResponse),
    Export,
    Sap,
}
