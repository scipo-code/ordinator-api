use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderText {
    pub order_system_status: String,
    pub order_user_status: String,
    pub order_description: String,
    pub operation_description: String,
    pub object_description: String,
    pub notes_1: String,
    pub notes_2: u32,
}

impl Default for OrderText {
    fn default() -> Self {
        OrderText {
            order_system_status: String::from(""),
            order_user_status: String::from(""),
            order_description: String::from(""),
            operation_description: String::from(""),
            object_description: String::from(""),
            notes_1: String::from(""),
            notes_2: 0,
        }
    }
}
