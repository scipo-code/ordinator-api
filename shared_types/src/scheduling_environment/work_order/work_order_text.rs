use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderText {
    pub order_system_status: String,
    pub order_user_status: String,
    pub order_description: String,
    pub operation_description: String,
    pub object_description: String,
    pub notes_1: String,
    pub notes_2: u64,
}
