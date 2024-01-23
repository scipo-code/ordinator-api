use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WorkOrderType {
    Wdf(WDFPriority),
    Wgn(WGNPriority),
    Wpm(WPMPriority),
    Other,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WDFPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WGNPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
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
