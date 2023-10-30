use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct OrderText {
    pub order_system_status: String,
    pub order_user_status: String,
    pub order_description: String,
    pub operation_description: String,
    pub object_description: String,
    pub notes_1: String,
    pub notes_2: u32,
}