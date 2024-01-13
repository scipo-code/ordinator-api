use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// This enum holds all the resources that are available needed to schedule work order.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, EnumIter)]
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
            _ => panic!("Invalid resource"),
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
        }
    }
}
