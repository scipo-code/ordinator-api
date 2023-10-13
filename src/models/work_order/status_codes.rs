use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StatusCodes {
    pub SMAT: bool,
    pub NMAT: bool,
    pub CMAT: bool,
    pub WMAT: bool,
    pub PMAT: bool,
    pub PCNF: bool,
    pub AWSC: bool,
    pub WELL: bool,
    pub SCH: bool,
    pub Unloading_Point: bool,
}