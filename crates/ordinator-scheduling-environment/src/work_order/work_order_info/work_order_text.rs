use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderText {
    pub order_system_status: Option<String>,
    pub order_user_status: Option<String>,
    pub order_description: String,
    pub operation_description: Option<String>,
    pub object_description: Option<String>,
    pub notes_1: Option<String>,
    pub notes_2: Option<u64>,
}

impl WorkOrderText {
    pub fn new(
        order_system_status: Option<String>,
        order_user_status: Option<String>,
        order_description: String,
        operation_description: Option<String>,
        object_description: Option<String>,
        notes_1: Option<String>,
        notes_2: Option<u64>,
    ) -> Self {
        Self {
            order_system_status,
            order_user_status,
            order_description,
            operation_description,
            object_description,
            notes_1,
            notes_2,
        }
    }
}
