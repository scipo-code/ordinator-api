use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub enum WorkOrderType {
    WDF(WDFPriority),
    WGN(WGNPriority),
    WPM(WPMPriority),
    Other,
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum WDFPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum WGNPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum WPMPriority {
    A,
    B,
    C,
    D,
}


impl WDFPriority {
    #[cfg(test)]
    pub fn new(priority: u32) -> Self {
        match priority {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => Self::Four,
        }
    }
}