use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct StatusCodes {
    pub material_status: MaterialStatus,
    pub pcnf: bool,
    pub awsc: bool,
    pub well: bool,
    pub sch: bool,
    pub sece: bool,
    pub unloading_point: bool,
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum MaterialStatus {
    Smat,
    Nmat,
    Cmat,
    Wmat,
    Pmat,
    Unknown
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

impl StatusCodes {

    #[cfg(test)]
    pub fn new_default() -> Self {
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