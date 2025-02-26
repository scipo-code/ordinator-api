use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use status_codes::{SystemStatusCodes, UserStatusCodes};

use crate::scheduling_environment::worker_environment::resources::Resources;

use super::operation::{ActivityNumber, Operation, Work};

pub mod status_codes;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    pub fixed: bool,
    pub vendor: bool,
    pub system_status_codes: SystemStatusCodes,
    pub user_status_codes: UserStatusCodes,
}

pub struct workorderanalyticbuilder {
    pub fixed: bool,
    pub vendor: bool,
    // TODO [ ]
    // You should make a builder for these if needed
    pub system_status_codes: SystemStatusCodes,
    // TODO [ ]
    // You should make a builder for these if needed
    pub user_status_codes: UserStatusCodes,
}

impl WorkOrderAnalyticBuilder {
    pub fn build(self, operations: HashMap<ActivityNumber, Operation>) -> WorkOrderAnalytic {
        WorkOrderAnalytic {
            fixed: todo!(),
            vendor: todo!(),
            system_status_codes: todo!(),
            user_status_codes: todo!(),
        }
    }
}

impl WorkOrderAnalytic {
    pub fn new(
        fixed: bool,
        vendor: bool,
        system_status_codes: SystemStatusCodes,
        user_status_codes: UserStatusCodes,
    ) -> Self {
        WorkOrderAnalytic {
            fixed,
            vendor,
            system_status_codes,
            user_status_codes,
        }
    }
}
