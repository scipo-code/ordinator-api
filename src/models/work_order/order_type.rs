use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum WorkOrderType {
    WDF(WDFPriority),
    WGN(WGNPriority),
    WPM(WPMPriority),
    Other,
}
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum WDFPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum WGNPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum WPMPriority {
    A,
    B,
    C,
    D,
}