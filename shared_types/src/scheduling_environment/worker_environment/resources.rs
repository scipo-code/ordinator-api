use chrono::NaiveTime;
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use strum_macros::EnumIter;

use crate::InputSupervisor;

/// This enum holds all the resources that are available needed to schedule work order.
#[derive(
    PartialOrd,
    Ord,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    EnumIter,
    clap::ValueEnum,
)]
pub enum Resources {
    #[serde(rename = "MTN-PIPF")]
    MtnPipf,
    #[serde(rename = "VEN-TURB")]
    VenTurb,
    #[serde(rename = "CON-VEN")]
    ConVen,
    #[serde(rename = "MTN-LAGG")]
    MtnLagg,
    #[serde(rename = "VEN-SCAF")]
    VenScaf,
    #[serde(rename = "MTN-ROPE")]
    MtnRope,
    #[serde(rename = "VEN-INSP")]
    VenInsp,
    #[serde(rename = "INP-SITE")]
    InpSite,
    #[serde(rename = "VEN-INST")]
    VenInst,
    #[serde(rename = "MAINONSH")]
    Mainonsh,
    #[serde(rename = "DRILLING")]
    Drilling,
    #[serde(rename = "WELLMAIN")]
    Wellmain,
    #[serde(rename = "WELLSUPV")]
    Wellsupv,
    #[serde(rename = "WELLTECH")]
    Welltech,
    #[serde(rename = "CON-ELEC")]
    ConElec,
    #[serde(rename = "CON-INPF")]
    ConInpf,
    #[serde(rename = "CON-INST")]
    ConInst,
    #[serde(rename = "CON-LAGG")]
    ConLagg,
    #[serde(rename = "CON-NDTI")]
    ConNdti,
    #[serde(rename = "CON-SCAF")]
    ConScaf,
    #[serde(rename = "CON-PAIN")]
    ConPain,
    #[serde(rename = "CON-RIGG")]
    ConRigg,
    #[serde(rename = "CON-ROPE")]
    ConRope,
    #[serde(rename = "CON-WELD")]
    ConWeld,
    #[serde(rename = "MTN-ROUS")]
    MtnRous,
    #[serde(rename = "MTN-CRAN")]
    MtnCran,
    #[serde(rename = "MTN-ELEC")]
    MtnElec,
    #[serde(rename = "MTN-INST")]
    MtnInst,
    #[serde(rename = "MTN-MECH")]
    MtnMech,
    #[serde(rename = "MTN-RIGG")]
    MtnRigg,
    #[serde(rename = "MTN-SCAF")]
    MtnScaf,
    #[serde(rename = "MTN-PAIN")]
    MtnPain,
    #[serde(rename = "MTN-TELE")]
    MtnTele,
    #[serde(rename = "MTN-TURB")]
    MtnTurb,
    #[serde(rename = "MEDIC")]
    Medic,
    #[serde(rename = "PRODLABO")]
    Prodlabo,
    #[serde(rename = "PRODTECH")]
    Prodtech,
    #[serde(rename = "MTN-SAT")]
    MtnSat,
    #[serde(rename = "VEN-ACCO")]
    VenAcco,
    #[serde(rename = "VEN-COMM")]
    VenComm,
    #[serde(rename = "VEN-CRAN")]
    VenCran,
    #[serde(rename = "VEN-ELEC")]
    VenElec,
    #[serde(rename = "VEN-HVAC")]
    VenHvac,
    #[serde(rename = "VEN-MECH")]
    VenMech,
    #[serde(rename = "VEN-METE")]
    VenMete,
    #[serde(rename = "VEN-SUBS")]
    VenSubs,
    #[serde(rename = "VEN-ROPE")]
    VenRope,
    #[serde(rename = "QAQCELEC")]
    Qaqcelec,
    #[serde(rename = "QAQCMECH")]
    Qaqcmech,
    #[serde(rename = "QAQCPAIN")]
    Qaqcpain,
    #[serde(rename = "PRODCCR")]
    Prodccr,
    #[serde(rename = "VEN-FFEQ")]
    VenFfeq,
    #[serde(rename = "CMP-RIGG")]
    CmpRigg,
    #[serde(rename = "CMP-SCAF")]
    CmpScaf,
    #[serde(rename = "CON-NPT")]
    ConNpt,
}

impl FromStr for Resources {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let resource = match s {
            "MTN-PIPF" => Resources::MtnPipf,
            "VEN-TURB" => Resources::VenTurb,
            "CON-VEN" => Resources::ConVen,
            "MTN-LAGG" => Resources::MtnLagg,
            "VEN-SCAF" => Resources::VenScaf,
            "MTN-ROPE" => Resources::MtnRope,
            "VEN-INSP" => Resources::VenInsp,
            "INP-SITE" => Resources::InpSite,
            "VEN-INST" => Resources::VenInst,
            "MAINONSH" => Resources::Mainonsh,
            "DRILLING" => Resources::Drilling,
            "WELLMAIN" => Resources::Wellmain,
            "WELLSUPV" => Resources::Wellsupv,
            "WELLTECH" => Resources::Welltech,
            "CON-ELEC" => Resources::ConElec,
            "CON-INPF" => Resources::ConInpf,
            "CON-INST" => Resources::ConInst,
            "CON-LAGG" => Resources::ConLagg,
            "CON-NDTI" => Resources::ConNdti,
            "CON-SCAF" => Resources::ConScaf,
            "CON-PAIN" => Resources::ConPain,
            "CON-RIGG" => Resources::ConRigg,
            "CON-ROPE" => Resources::ConRope,
            "CON-WELD" => Resources::ConWeld,
            "MTN-ROUS" => Resources::MtnRous,
            "MTN-CRAN" => Resources::MtnCran,
            "MTN-ELEC" => Resources::MtnElec,
            "MTN-INST" => Resources::MtnInst,
            "MTN-MECH" => Resources::MtnMech,
            "MTN-RIGG" => Resources::MtnRigg,
            "MTN-SCAF" => Resources::MtnScaf,
            "MTN-PAIN" => Resources::MtnPain,
            "MTN-TELE" => Resources::MtnTele,
            "MTN-TURB" => Resources::MtnTurb,
            "MEDIC" => Resources::Medic,
            "PRODLABO" => Resources::Prodlabo,
            "PRODTECH" => Resources::Prodtech,
            "MTN-SAT" => Resources::MtnSat,
            "VEN-ACCO" => Resources::VenAcco,
            "VEN-COMM" => Resources::VenComm,
            "VEN-CRAN" => Resources::VenCran,
            "VEN-ELEC" => Resources::VenElec,
            "VEN-HVAC" => Resources::VenHvac,
            "VEN-MECH" => Resources::VenMech,
            "VEN-METE" => Resources::VenMete,
            "VEN-SUBS" => Resources::VenSubs,
            "VEN-ROPE" => Resources::VenRope,
            "QAQCELEC" => Resources::Qaqcelec,
            "QAQCMECH" => Resources::Qaqcmech,
            "QAQCPAIN" => Resources::Qaqcpain,
            "PRODCCR" => Resources::Prodccr,
            "VEN-FFEQ" => Resources::VenFfeq,
            "CMP-RIGG" => Resources::CmpRigg,
            "CMP-SCAF" => Resources::CmpScaf,
            "CON-NPT" => Resources::ConNpt,
            unknown => return Err(format!("Could not parse Resource: {}", unknown)),
        };
        Ok(resource)
    }
}

impl Resources {
    pub fn variant_name(&self) -> String {
        match self {
            Resources::Medic => "MEDIC".to_string(),
            Resources::MtnCran => "MTN-CRAN".to_string(),
            Resources::MtnElec => "MTN-ELEC".to_string(),
            Resources::MtnInst => "MTN-INST".to_string(),
            Resources::MtnLagg => "MTN-LAGG".to_string(),
            Resources::MtnMech => "MTN-MECH".to_string(),
            Resources::MtnPain => "MTN-PAIN".to_string(),
            Resources::MtnPipf => "MTN-PIPF".to_string(),
            Resources::MtnRigg => "MTN-RIGG".to_string(),
            Resources::MtnRope => "MTN-ROPE".to_string(),
            Resources::MtnRous => "MTN-ROUS".to_string(),
            Resources::MtnSat => "MTN-SAT".to_string(),
            Resources::MtnScaf => "MTN-SCAF".to_string(),
            Resources::MtnTele => "MTN-TELE".to_string(),
            Resources::MtnTurb => "MTN-TURB".to_string(),
            Resources::InpSite => "INP-SITE".to_string(),
            Resources::Prodlabo => "PRODLABO".to_string(),
            Resources::Prodtech => "PRODTECH".to_string(),
            Resources::VenAcco => "VEN-ACCO".to_string(),
            Resources::VenComm => "VEN-COMM".to_string(),
            Resources::VenCran => "VEN-CRAN".to_string(),
            Resources::VenElec => "VEN-ELEC".to_string(),
            Resources::VenHvac => "VEN-HVAC".to_string(),
            Resources::VenInsp => "VEN-INSP".to_string(),
            Resources::VenInst => "VEN-INST".to_string(),
            Resources::VenMech => "VEN-MECH".to_string(),
            Resources::VenMete => "VEN-METE".to_string(),
            Resources::VenRope => "VEN-ROPE".to_string(),
            Resources::VenScaf => "VEN-SCAF".to_string(),
            Resources::VenSubs => "VEN-SUBS".to_string(),
            Resources::Qaqcelec => "QAQCELEC".to_string(),
            Resources::Qaqcmech => "QAQCMECH".to_string(),
            Resources::Qaqcpain => "QAQCPAIN".to_string(),
            Resources::Wellsupv => "WELLSUPV".to_string(),
            Resources::VenTurb => "VEN-TURB".to_string(),
            Resources::ConVen => "CON-VEN".to_string(),
            Resources::Mainonsh => "MAINONSH".to_string(),
            Resources::Drilling => "DRILLING".to_string(),
            Resources::Wellmain => "WELLMAIN".to_string(),
            Resources::Welltech => "WELLTECH".to_string(),
            Resources::ConElec => "CON-ELEC".to_string(),
            Resources::ConInpf => "CON-INPF".to_string(),
            Resources::ConInst => "CON-INST".to_string(),
            Resources::ConLagg => "CON-LAGG".to_string(),
            Resources::ConNdti => "CON-NDTI".to_string(),
            Resources::ConScaf => "CON-SCAF".to_string(),
            Resources::ConPain => "CON-PAIN".to_string(),
            Resources::ConRigg => "CON-RIGG".to_string(),
            Resources::ConRope => "CON-ROPE".to_string(),
            Resources::ConWeld => "CON-WELD".to_string(),
            Resources::Prodccr => "PRODCCR".to_string(),
            Resources::VenFfeq => "VEN-FFEQ".to_string(),
            Resources::CmpRigg => "CMP-RIGG".to_string(),
            Resources::CmpScaf => "CMP-SCAF".to_string(),
            Resources::ConNpt => "CON-NPT".to_string(),
        }
    }

    pub fn is_ven_variant(&self) -> bool {
        matches!(
            self,
            Resources::VenAcco
                | Resources::VenComm
                | Resources::VenCran
                | Resources::VenElec
                | Resources::VenHvac
                | Resources::VenInsp
                | Resources::VenInst
                | Resources::VenMech
                | Resources::VenMete
                | Resources::VenRope
                | Resources::VenScaf
                | Resources::VenSubs
        )
    }
    pub fn is_fmc(&self) -> bool {
        matches!(
            self,
            Self::MtnRope
                | Self::MtnScaf
                | Self::MtnRigg
                | Self::MtnLagg
                | Self::MtnPipf
                | Self::MtnPain
        )
    }
}

impl Display for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.variant_name())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Id(pub String, pub Vec<Resources>, pub Option<InputSupervisor>);

impl Id {
    pub fn new(
        id_employee: String,
        resources: Vec<Resources>,
        supervisor: Option<InputSupervisor>,
    ) -> Self {
        Id(id_employee, resources, supervisor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, EnumIter, clap::ValueEnum)]
pub enum Shift {
    Day,
    Night,
}

impl Shift {
    pub fn generate_time_intervals(&self) -> (NaiveTime, NaiveTime) {
        match self {
            Shift::Day => (
                NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            ),
            Shift::Night => (
                NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            ),
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {:?}", self.0, self.1)
    }
}

impl IntoExcelData for Resources {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.variant_name();
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.variant_name();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
