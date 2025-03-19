use std::fmt::Display;
use std::fmt::{self};

use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicStatusMessage {
    General,
    Period(String),
    WorkOrder(WorkOrderNumber),
}

impl StrategicStatusMessage {
    pub fn new_period(period: String) -> Self {
        Self::Period(period)
    }
}

impl Display for StrategicStatusMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategicStatusMessage::General => write!(f, "general"),
            StrategicStatusMessage::Period(period) => write!(f, "period: {}", period),
            StrategicStatusMessage::WorkOrder(work_order_number) => {
                write!(f, "{:?}", work_order_number)
            }
        }
    }
}
