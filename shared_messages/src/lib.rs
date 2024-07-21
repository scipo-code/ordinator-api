pub mod agent_error;
pub mod operational;
pub mod orchestrator;
pub mod scheduling_environment;
pub mod strategic;
pub mod supervisor;
pub mod tactical;
use std::fmt::{self, Display};

use actix::prelude::*;
use clap::{Subcommand, ValueEnum};
use operational::{OperationalRequest, OperationalResponse, TomlOperationalConfiguration};
use orchestrator::{OrchestratorRequest, OrchestratorResponse};
use scheduling_environment::worker_environment::resources::Resources;
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

#[derive(
    steel_derive::Steel, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Clone, ValueEnum,
)]
pub enum Asset {
    DF,
    DM,
    DE,
    GO,
    HB,
    HC,
    HD,
    HW,
    KR,
    RO,
    RF,
    SK,
    SV,
    TE,
    TS,
    VA,
    VB,
    Unknown,
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Asset::DF => write!(f, "DF"),
            Asset::DM => write!(f, "DM"),
            Asset::DE => write!(f, "DE"),
            Asset::GO => write!(f, "GO"),
            Asset::HB => write!(f, "HB"),
            Asset::HC => write!(f, "HC"),
            Asset::HD => write!(f, "HD"),
            Asset::HW => write!(f, "HW"),
            Asset::KR => write!(f, "KR"),
            Asset::RO => write!(f, "RO"),
            Asset::RF => write!(f, "RF"),
            Asset::SK => write!(f, "SK"),
            Asset::SV => write!(f, "SV"),
            Asset::TE => write!(f, "TE"),
            Asset::TS => write!(f, "TS"),
            Asset::VA => write!(f, "VA"),
            Asset::VB => write!(f, "VB"),
            Asset::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Asset {
    pub fn new_from_string(asset_string: String) -> Asset {
        match asset_string.as_str() {
            "DF" => Asset::DF,
            "DM" => Asset::DM,
            "DE" => Asset::DE,
            "GO" => Asset::GO,
            "HB" => Asset::HB,
            "HC" => Asset::HC,
            "HD" => Asset::HD,
            "HW" => Asset::HW,
            "KR" => Asset::KR,
            "RO" => Asset::RO,
            "RF" => Asset::RF,
            "SK" => Asset::SK,
            "SV" => Asset::SV,
            "TE" => Asset::TE,
            "TS" => Asset::TS,
            "VA" => Asset::VA,
            "VB" => Asset::VB,
            _ => Asset::Unknown,
        }
    }
}

#[derive(Deserialize)]
pub struct TomlResources {
    pub medic: f64,
    pub mtncran: f64,
    pub mtnelec: f64,
    pub mtninst: f64,
    pub mtnlagg: f64,
    pub mtnmech: f64,
    pub mtnpain: f64,
    pub mtnpipf: f64,
    pub mtnrigg: f64,
    pub mtnrope: f64,
    pub mtnrous: f64,
    pub mtnsat: f64,
    pub mtnscaf: f64,
    pub mtntele: f64,
    pub mtnturb: f64,
    pub inpsite: f64,
    pub prodlabo: f64,
    pub prodtech: f64,
    pub venacco: f64,
    pub vencomm: f64,
    pub vencran: f64,
    pub venelec: f64,
    pub venhvac: f64,
    pub veninsp: f64,
    pub veninst: f64,
    pub venmech: f64,
    pub venmete: f64,
    pub venrope: f64,
    pub venscaf: f64,
    pub vensubs: f64,
    pub qaqcelec: f64,
    pub qaqcmech: f64,
    pub qaqcpain: f64,
    pub wellsupv: f64,
}

#[derive(Deserialize, Debug)]
pub struct TomlAgents {
    pub operational: Vec<TomlOperational>,
}

#[derive(Deserialize, Debug)]
pub struct TomlOperational {
    pub id: String,
    pub resources: TomlResourcesArray,
    pub hours_per_day: f64,
    pub operational_configuration: TomlOperationalConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct TomlResourcesArray {
    pub resources: Vec<Resources>,
}

#[derive(Debug, Serialize)]
pub enum AlgorithmState<T> {
    Feasible,
    Infeasible(T),
}

impl<T> AlgorithmState<T> {
    pub fn infeasible_cases_mut(&mut self) -> Option<&mut T> {
        match self {
            AlgorithmState::Feasible => None,
            AlgorithmState::Infeasible(infeasible_cases) => Some(infeasible_cases),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ConstraintState<Reason> {
    Feasible,
    Infeasible(Reason),
    Undetermined,
}

impl<Reason> fmt::Display for ConstraintState<Reason>
where
    Reason: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintState::Feasible => write!(f, "FEASIBLE"),
            ConstraintState::Infeasible(reason) => write!(f, "{}", reason),
            ConstraintState::Undetermined => write!(f, "Constraint is not determined yet"),
        }
    }
}
pub enum LoadOperation {
    Add,
    Sub,
}

#[cfg(test)]
mod tests {

    use crate::{scheduling_environment::worker_environment::resources::Resources, TomlAgents};

    #[test]
    fn test_toml_operational_parsing() {
        let toml_operational_string = r#"
            [[operational]]
            id = "OP-01-001"
            resources.resources = ["MTN-ELEC" ]
            hours_per_day = 6.0
            operational_configuration.shift_interval = [07:00:00, 19:00:00]
            operational_configuration.break_interval = [11:00:00, 12:00:00]
            operational_configuration.toolbox_interval = [07:00:00, 08:00:00]
            operational_configuration.availability.start_date = 2024-05-16T07:00:00Z
            operational_configuration.availability.end_date = 2024-05-30T15:00:00Z
        "#;

        let toml_agents: TomlAgents = toml::from_str(&toml_operational_string).unwrap();

        assert_eq!(toml_agents.operational[0].id, "OP-01-001".to_string());

        assert_eq!(
            toml_agents.operational[0].resources.resources,
            [Resources::MtnElec]
        );

        assert_eq!(
            toml_agents.operational[0]
                .operational_configuration
                .shift_interval
                .start
                .time
                .unwrap(),
            toml::value::Time {
                hour: 7,
                minute: 0,
                second: 0,
                nanosecond: 0
            }
        );
        assert_eq!(
            toml_agents.operational[0]
                .operational_configuration
                .shift_interval
                .end
                .time
                .unwrap(),
            toml::value::Time {
                hour: 19,
                minute: 0,
                second: 0,
                nanosecond: 0
            }
        );
    }
}
