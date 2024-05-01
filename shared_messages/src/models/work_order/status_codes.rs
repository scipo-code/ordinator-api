use std::fmt::{self, Display};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StatusCodes {
    pub material_status: MaterialStatus,
    pub pcnf: bool,
    pub awsc: bool,
    pub well: bool,
    pub sch: bool,
    pub sece: bool,
    pub unloading_point: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum MaterialStatus {
    Smat,
    Nmat,
    Cmat,
    Wmat,
    Pmat,
    Unknown,
}

impl MaterialStatus {
    pub fn from_status_code_string(status_codes_string: &str) -> Self {
        // Define individual patterns for clarity and precise matching
        let patterns = vec![
            ("SMAT", MaterialStatus::Smat),
            ("NMAT", MaterialStatus::Nmat),
            ("CMAT", MaterialStatus::Cmat),
            ("WMAT", MaterialStatus::Wmat),
            ("PMAT", MaterialStatus::Pmat),
        ];

        // Check each pattern to see if it matches the status code string
        for (pattern, status) in patterns {
            if Regex::new(pattern).unwrap().is_match(status_codes_string) {
                return status;
            }
        }

        MaterialStatus::Unknown
        // If no patterns match, return the Unknown variant
    }
}

impl Display for MaterialStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaterialStatus::Smat => write!(f, "SMAT"),
            MaterialStatus::Nmat => write!(f, "NMAT"),
            MaterialStatus::Cmat => write!(f, "CMAT"),
            MaterialStatus::Wmat => write!(f, "WMAT"),
            MaterialStatus::Pmat => write!(f, "PMAT"),
            MaterialStatus::Unknown => write!(f, "----"),
        }
    }
}

impl Display for StatusCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            " | {} | {} | {} | {} | {} | {} | {} |",
            self.material_status,
            if self.pcnf { "PCNF" } else { "" },
            if self.awsc { "AWSC" } else { "" },
            if self.well { "WELL" } else { "" },
            if self.sch { "SCH" } else { "" },
            if self.sece { "SECE" } else { "" },
            if self.unloading_point {
                "UNLOADING POINT"
            } else {
                ""
            }
        )
    }
}

impl Default for StatusCodes {
    fn default() -> Self {
        StatusCodes {
            material_status: MaterialStatus::Unknown,
            pcnf: false,
            awsc: false,
            well: false,
            sch: false,
            sece: false,
            unloading_point: false,
        }
    }
}
