pub mod agent_error;
pub mod orchestrator;
pub mod resources;
pub mod strategic;
pub mod tactical;

use actix::prelude::*;
use clap::{Subcommand, ValueEnum};
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

pub struct SolutionExportMessage {}

impl Message for SolutionExportMessage {
    type Result = String;
}

#[derive(Deserialize, Serialize, Debug, Clone, ValueEnum)]
pub enum LevelOfDetail {
    Normal,
    Verbose,
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn to_level_string(&self) -> String {
        match self {
            LogLevel::Trace => "trace".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Info => "info".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Error => "error".to_string(),
        }
    }
}
