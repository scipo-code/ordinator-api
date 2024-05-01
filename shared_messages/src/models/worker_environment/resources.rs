use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, EnumIter, clap::ValueEnum)]
pub enum MainResources {
    MtnMech,
    MtnElec,
    MtnInst,
    MtnRope,
    MtnPipf,
    MtnCran,
    Prodtech,
    MtnTele,
    MtnTurb,
    MtnPain,
    VenInsp,
    Wellsupv,
    InpSite,
    MtnLagg,
    MtnRous,
    VenMech,
    MtnSat,
    Qaqcmech,
    Prodlabo,
    MtnScaf,
    Wellmain,
    VenInst,
    VenElec,
    VenSubs,
    MtnRigg,
    VenCran,
    VenRope,
    Welltech,
    VenComm,
    Qaqcelec,
    Medic,
    Unknown,
}

impl MainResources {
    pub fn is_fmc(&self) -> bool {
        match self {
            Self::MtnRope => true,
            Self::MtnScaf => true,
            Self::MtnRigg => true,
            Self::MtnLagg => true,
            Self::MtnPipf => true,
            Self::MtnPain => true,
            _ => false,
        }
    }
}

/// This enum holds all the resources that are available needed to schedule work order.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, EnumIter, clap::ValueEnum)]
pub enum Resources {
    #[serde(rename = "MEDIC")]
    Medic,
    #[serde(rename = "MTN-CRAN")]
    MtnCran,
    #[serde(rename = "MTN-ELEC")]
    MtnElec,
    #[serde(rename = "MTN-INST")]
    MtnInst,
    #[serde(rename = "MTN-LAGG")]
    MtnLagg,
    #[serde(rename = "MTN-MECH")]
    MtnMech,
    #[serde(rename = "MTN-PAIN")]
    MtnPain,
    #[serde(rename = "MTN-PIPF")]
    MtnPipf,
    #[serde(rename = "MTN-RIGG")]
    MtnRigg,
    #[serde(rename = "MTN-ROPE")]
    MtnRope,
    #[serde(rename = "MTN-ROUS")]
    MtnRous,
    #[serde(rename = "MTN-SAT")]
    MtnSat,
    #[serde(rename = "MTN-SCAF")]
    MtnScaf,
    #[serde(rename = "MTN-TELE")]
    MtnTele,
    #[serde(rename = "MTN-TURB")]
    MtnTurb,
    #[serde(rename = "INP-SITE")]
    InpSite,
    #[serde(rename = "PRODLABO")]
    Prodlabo,
    #[serde(rename = "PRODTECH")]
    Prodtech,
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
    #[serde(rename = "VEN-INSP")]
    VenInsp,
    #[serde(rename = "VEN-INST")]
    VenInst,
    #[serde(rename = "VEN-MECH")]
    VenMech,
    #[serde(rename = "VEN-METE")]
    VenMete,
    #[serde(rename = "VEN-ROPE")]
    VenRope,
    #[serde(rename = "VEN-SCAF")]
    VenScaf,
    #[serde(rename = "VEN-SUBS")]
    VenSubs,
    #[serde(rename = "QAQCELEC")]
    QaqcElec,
    #[serde(rename = "QAQCMECH")]
    QaqcMech,
    #[serde(rename = "QAQCPAIN")]
    QaqcPain,
    #[serde(rename = "WellSupv")]
    WellSupv,
}

impl Resources {
    pub fn new_from_string(resource: String) -> Self {
        match resource.as_str() {
            "MEDIC" => Resources::Medic,
            "MTN-CRAN" => Resources::MtnCran,
            "MTN-ELEC" => Resources::MtnElec,
            "MTN-INST" => Resources::MtnInst,
            "MTN-LAGG" => Resources::MtnLagg,
            "MTN-MECH" => Resources::MtnMech,
            "MTN-PAIN" => Resources::MtnPain,
            "MTN-PIPF" => Resources::MtnPipf,
            "MTN-RIGG" => Resources::MtnRigg,
            "MTN-ROPE" => Resources::MtnRope,
            "MTN-ROUS" => Resources::MtnRous,
            "MTN-SAT" => Resources::MtnSat,
            "MTN-SCAF" => Resources::MtnScaf,
            "MTN-TELE" => Resources::MtnTele,
            "MTN-TURB" => Resources::MtnTurb,
            "INP-SITE" => Resources::InpSite,
            "PRODLABO" => Resources::Prodlabo,
            "PRODTECH" => Resources::Prodtech,
            "VEN-ACCO" => Resources::VenAcco,
            "VEN-COMM" => Resources::VenComm,
            "VEN-CRAN" => Resources::VenCran,
            "VEN-ELEC" => Resources::VenElec,
            "VEN-HVAC" => Resources::VenHvac,
            "VEN-INSP" => Resources::VenInsp,
            "VEN-INST" => Resources::VenInst,
            "VEN-MECH" => Resources::VenMech,
            "VEN-METE" => Resources::VenMete,
            "VEN-ROPE" => Resources::VenRope,
            "VEN-SCAF" => Resources::VenScaf,
            "VEN-SUBS" => Resources::VenSubs,
            "QAQCELEC" => Resources::QaqcElec,
            "QAQCMECH" => Resources::QaqcMech,
            "QAQCPAIN" => Resources::QaqcPain,
            "WELLSUPV" => Resources::WellSupv,
            _ => Resources::WellSupv,
        }
    }

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
            Resources::QaqcElec => "QAQCELEC".to_string(),
            Resources::QaqcMech => "QAQCMECH".to_string(),
            Resources::QaqcPain => "QAQCPAIN".to_string(),
            Resources::WellSupv => "WELLSUPV".to_string(),
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
}

impl Display for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.variant_name())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Id(pub String, pub Vec<Resources>, pub Option<MainResources>);

impl Id {
    pub fn new(
        id: String,
        resources: Vec<Resources>,
        main_resources: Option<MainResources>,
    ) -> Self {
        Id(id, resources, main_resources)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {:?}", self.0, self.1)
    }
}

impl MainResources {
    pub fn new_from_string(resource: String) -> Self {
        match resource.as_str() {
            "MTN-MECH" => MainResources::MtnMech,
            "MTN-ELEC" => MainResources::MtnElec,
            "MTN-INST" => MainResources::MtnInst,
            "MTN-ROPE" => MainResources::MtnRope,
            "MTN-PIPF" => MainResources::MtnPipf,
            "MTN-CRAN" => MainResources::MtnCran,
            "PRODTECH" => MainResources::Prodtech,
            "MTN-TELE" => MainResources::MtnTele,
            "MTN-TURB" => MainResources::MtnTurb,
            "MTN-PAIN" => MainResources::MtnPain,
            "VEN-INSP" => MainResources::VenInsp,
            "WELLSUPV" => MainResources::Wellsupv,
            "INP-SITE" => MainResources::InpSite,
            "MTN-LAGG" => MainResources::MtnLagg,
            "MTN-ROUS" => MainResources::MtnRous,
            "VEN-MECH" => MainResources::VenMech,
            "MTN-SAT" => MainResources::MtnSat,
            "QAQCMECH" => MainResources::Qaqcmech,
            "PRODLABO" => MainResources::Prodlabo,
            "MTN-SCAF" => MainResources::MtnScaf,
            "WELLMAIN" => MainResources::Wellmain,
            "VEN-INST" => MainResources::VenInst,
            "VEN-ELEC" => MainResources::VenElec,
            "VEN-SUBS" => MainResources::VenSubs,
            "MTN-RIGG" => MainResources::MtnRigg,
            "VEN-CRAN" => MainResources::VenCran,
            "VEN-ROPE" => MainResources::VenRope,
            "WELLTECH" => MainResources::Welltech,
            "VEN-COMM" => MainResources::VenComm,
            "QAQCELEC" => MainResources::Qaqcelec,
            "MEDIC" => MainResources::Medic,
            _ => MainResources::Unknown,
        }
    }
}
