// WARN
// There is a lot of duplication in all these shared types. I think that the
// best approach is to create something that will allow us to work with the
// API types with as little friction as possible.
pub mod agents;
pub mod configuration;
pub mod orchestrator;
pub mod scheduling_environment;

use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use clap::{Subcommand, ValueEnum};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use agents::operational::{OperationalConfiguration, OperationalRequest, OperationalResponse};
use orchestrator::{OrchestratorRequest, OrchestratorResponse};
use rust_xlsxwriter::IntoExcelData;
use scheduling_environment::{
    time_environment::{day::Day, period::Period},
    work_order::{WorkOrderActivity, WorkOrderNumber},
    worker_environment::resources::{Id, Resources},
};

use agents::strategic::{StrategicRequest, StrategicResponse};
use agents::supervisor::{SupervisorRequest, SupervisorResponse};
use agents::tactical::{TacticalRequest, TacticalResponse};
use serde::{Deserialize, Serialize};

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

pub struct SolutionExportMessage;

#[derive(Debug, Clone)]
pub enum ReasonForNotScheduling {
    Scheduled(Period),
    Unknown(String),
}

impl IntoExcelData for ReasonForNotScheduling {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self {
            ReasonForNotScheduling::Scheduled(period) => period.period_string(),
            ReasonForNotScheduling::Unknown(unknown) => unknown,
        };
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self {
            ReasonForNotScheduling::Scheduled(period) => period.period_string(),
            ReasonForNotScheduling::Unknown(unknown) => unknown,
        };
        worksheet.write_string_with_format(row, col, value, format)
    }
}

#[derive(Clone)]
pub enum AgentExports {
    // TODO
    // This Option should be changed into the reason is
    Strategic(HashMap<WorkOrderNumber, Option<Period>>),
    Tactical(HashMap<WorkOrderActivity, Day>),
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

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Clone, ValueEnum, EnumIter)]
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
    Test,
}

#[derive(Serialize)]
pub struct AssetNames {
    value: String,
    label: String,
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

    pub fn convert_to_asset_names() -> Vec<AssetNames> {
        let mut vec = Vec::new();
        for asset in Asset::iter() {
            let asset_name = AssetNames {
                value: asset.to_string(),
                label: asset.to_string(),
            };
            vec.push(asset_name);
        }
        vec
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

// WARN
// This type if for initializing data based on the configuration
// .toml files. It should not be used as the structure for the
// `WorkerEnvironments` that is inappropriate.
// Good, so this type is an `API` types. And all API types
// should be located together. I think that is the best approach
// here.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ActorSpecifications {
    pub supervisors: Vec<InputSupervisor>,
    // QUESTION
    // Why not just store the OperationalParameters here?
    // Hmm... because the WorkOrders should not be part of this
    // what about the options? The options should be defined in
    // a separate config file
    // TODO [] Make separate config files for options
    pub operational: Vec<InputOperational>,
}

impl From<ActorSpecifications> for AgentEnvironment {
    fn from(value: ActorSpecifications) -> Self {
        let operational = value
            .operational
            .into_iter()
            .map(|io| {
                let id = Id::new(&io.id, vec![], io.assets);
                let operational_config = OperationalConfigurationAll {
                    id: id.clone(),
                    hours_per_day: io.hours_per_day,
                    operational_configuration: io.operational_configuration,
                };

                (id, operational_config)
            })
            .collect();

        let supervisor = value
            .supervisors
            .into_iter()
            .map(|is| {
                // The umber of supervisor periods is misleading.
                let id = Id::new(&is.id, vec![], is.assets);
                let supervisor_config = SupervisorConfigurationAll {
                    id: id.clone(),
                    number_of_supervisor_periods: is.number_of_supervisor_periods,
                };
                (id.clone(), supervisor_config)
            })
            .collect();

        Self {
            operational,
            supervisor,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AgentEnvironment {
    pub operational: HashMap<Id, OperationalConfigurationAll>,
    pub supervisor: HashMap<Id, SupervisorConfigurationAll>,
}

// WARN
// You should never be able to clone this.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationalConfigurationAll {
    pub id: Id,
    hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
}

impl OperationalConfigurationAll {
    pub fn new(
        id: Id,
        hours_per_day: f64,
        operational_configuration: OperationalConfiguration,
    ) -> Self {
        Self {
            id,
            hours_per_day,
            operational_configuration,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfigurationAll {
    id: Id,
    // FIX
    // This information is found in two different places. That is an
    // error that has to be fixed.
    number_of_supervisor_periods: u64,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct InputSupervisor {
    pub id: String,
    pub resource: Option<Resources>,
    pub number_of_supervisor_periods: u64,
    pub assets: Vec<Asset>,
}

pub type OperationalId = String;

// I think that this should be refactored. You do not really understand
// why it is that way.
//
// You should make a new type here. One that does the best it
// can to be what we need this. `From<InputOperational> for OperationalConfiguration`
// is needed!
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputOperational {
    pub id: OperationalId,
    pub resources: Vec<Resources>,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
    pub assets: Vec<Asset>,
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

    use crate::{
        scheduling_environment::worker_environment::resources::Resources, ActorSpecifications,
    };

    #[test]
    fn test_toml_operational_parsing() {
        let toml_operational_string = r#"
            [[supervisors]]
            id = "main"
            number_of_supervisAgentEnvironmentr_periods = 3

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
            operational_configuration.availability.finish_date = "2024-12-15T15:00:00Z"
        "#;

        let system_agents: ActorSpecifications = toml::from_str(toml_operational_string).unwrap();

        assert_eq!(system_agents.operational[0].id, "OP-01-001".to_string());

        assert_eq!(system_agents.operational[0].resources, [Resources::MtnElec]);

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
