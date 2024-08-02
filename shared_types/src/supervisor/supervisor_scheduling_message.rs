use serde::{Deserialize, Serialize};

use crate::scheduling_environment::{
    work_order::WorkOrderActivity, worker_environment::resources::Id,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupervisorSchedulingMessage {
    pub work_order_activity: WorkOrderActivity,
    pub id_operational: Id,
}

impl SupervisorSchedulingMessage {
    pub fn new(work_order_activity: WorkOrderActivity, id_operational: Id) -> Self {
        Self {
            work_order_activity,
            id_operational,
        }
    }
}
