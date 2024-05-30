use serde::{Deserialize, Serialize};

use super::OperationalId;

#[derive(Deserialize, Serialize, Debug)]
pub enum OperationalStatusRequest {
    General,
}
