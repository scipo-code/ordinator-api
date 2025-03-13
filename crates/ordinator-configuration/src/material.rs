use serde::Deserialize;

#[derive(Deserialize)]
pub struct MaterialToPeriod {
    pub nmat: usize,
    pub smat: usize,
    pub cmat: usize,
    pub pmat: usize,
    pub wmat: usize,
}
