use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::scheduling_environment::work_order::WorkOrderNumber;

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
