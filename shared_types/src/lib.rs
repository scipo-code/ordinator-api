pub mod operational;
pub mod orchestrator;
pub mod scheduling_environment;
pub mod strategic;
pub mod supervisor;
pub mod tactical;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use actix::prelude::*;
use clap::{Subcommand, ValueEnum};

use operational::{OperationalConfiguration, OperationalRequest, OperationalResponse};
use orchestrator::{OrchestratorRequest, OrchestratorResponse};
use scheduling_environment::{
    time_environment::{day::Day, period::Period},
    work_order::{WorkOrderActivity, WorkOrderNumber},
    worker_environment::resources::Resources,
};

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

pub struct SolutionExportMessage;

#[derive(Clone)]
pub enum AgentExports {
    Strategic(HashMap<WorkOrderNumber, Period>),
    Tactical(HashMap<WorkOrderActivity, Day>),
}

impl Message for SolutionExportMessage {
    type Result = Option<AgentExports>;
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

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Clone, ValueEnum)]
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
    pub fn new_from_string(asset_string: &str) -> Option<Asset> {
        match asset_string {
            "DF" => Some(Asset::DF),
            "DM" => Some(Asset::DM),
            "DE" => Some(Asset::DE),
            "GO" => Some(Asset::GO),
            "HB" => Some(Asset::HB),
            "HC" => Some(Asset::HC),
            "HD" => Some(Asset::HD),
            "HW" => Some(Asset::HW),
            "KR" => Some(Asset::KR),
            "RO" => Some(Asset::RO),
            "RF" => Some(Asset::RF),
            "SK" => Some(Asset::SK),
            "SV" => Some(Asset::SV),
            "TE" => Some(Asset::TE),
            "TS" => Some(Asset::TS),
            "VA" => Some(Asset::VA),
            "VB" => Some(Asset::VB),
            _ => None,
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemAgents {
    pub supervisors: Vec<InputSupervisor>,
    pub operational: Vec<InputOperational>,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct InputSupervisor {
    pub id: String,
    pub resource: Option<Resources>,
    pub number_of_supervisor_periods: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputOperational {
    pub id: String,
    pub resources: TomlResourcesArray,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    use chrono::NaiveTime;

    use crate::{scheduling_environment::worker_environment::resources::Resources, SystemAgents};

    #[test]
    fn test_toml_operational_parsing() {
        let toml_operational_string = r#"
            [[supervisors]]
            id = "main"
            number_of_supervisor_periods = 3

            # [[supervisors]]
            # id = "supervisor-second"
            ################################
            ###          MTN-ELEC        ###
            ################################
            [[operational]]
            id = "OP-01-001"
            resources.resources = ["MTN-ELEC" ]
            hours_per_day = 6.0
            operational_configuration.off_shift_interval = { start = "19:00:00",  end = "07:00:00" }
            operational_configuration.break_interval = { start = "11:00:00", end = "12:00:00" }
            operational_configuration.toolbox_interval = { start = "07:00:00", end = "08:00:00" }
            operational_configuration.availability.start_date = "2024-12-02T07:00:00Z"
            operational_configuration.availability.end_date = "2024-12-15T15:00:00Z"
        "#;

        let system_agents: SystemAgents = toml::from_str(toml_operational_string).unwrap();

        assert_eq!(system_agents.operational[0].id, "OP-01-001".to_string());

        assert_eq!(
            system_agents.operational[0].resources.resources,
            [Resources::MtnElec]
        );

        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .start,
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        );
        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .end,
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
    }
}
