use serde::{Deserialize, Serialize};

use crate::scheduling_environment::{
    work_order::{operation::ActivityNumber, WorkOrderNumber},
    worker_environment::resources::Id,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupervisorSchedulingMessage {
    pub work_order_number: WorkOrderNumber,
    pub activity_number: ActivityNumber,
    pub id_operational: Id,
}

impl SupervisorSchedulingMessage {
    pub fn new(
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        id_operational: Id,
    ) -> Self {
        Self {
            work_order_number,
            activity_number,
            id_operational,
        }
    }
}
