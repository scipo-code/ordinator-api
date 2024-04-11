pub mod agent_error;
pub mod orchestrator;
pub mod resources;
pub mod strategic;
pub mod tactical;

use std::fmt::Display;

use actix::prelude::*;
use clap::{Subcommand, ValueEnum};
use orchestrator::OrchestratorRequest;
use serde::{Deserialize, Serialize};
use strategic::StrategicRequest;
use tactical::TacticalRequest;

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
    pub fn new_from_string(asset_string: &str) -> Asset {
        match asset_string {
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
    pub Medic: f64,
    pub MtnCran: f64,
    pub MtnElec: f64,
    pub MtnInst: f64,
    pub MtnLagg: f64,
    pub MtnMech: f64,
    pub MtnPain: f64,
    pub MtnPipf: f64,
    pub MtnRigg: f64,
    pub MtnRope: f64,
    pub MtnRous: f64,
    pub MtnSat: f64,
    pub MtnScaf: f64,
    pub MtnTele: f64,
    pub MtnTurb: f64,
    pub InpSite: f64,
    pub Prodlabo: f64,
    pub Prodtech: f64,
    pub VenAcco: f64,
    pub VenComm: f64,
    pub VenCran: f64,
    pub VenElec: f64,
    pub VenHvac: f64,
    pub VenInsp: f64,
    pub VenInst: f64,
    pub VenMech: f64,
    pub VenMete: f64,
    pub VenRope: f64,
    pub VenScaf: f64,
    pub VenSubs: f64,
    pub QaqcElec: f64,
    pub QaqcMech: f64,
    pub QaqcPain: f64,
    pub WellSupv: f64,
}
