use serde::Deserialize;
use serde::Serialize;
// TODO [ ]
// This should be moved, the question is whether we should make one or two types

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialToPeriod {
    pub nmat: usize,
    pub smat: usize,
    pub cmat: usize,
    pub pmat: usize,
    pub wmat: usize,
}
