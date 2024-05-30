use serde::{Deserialize, Serialize};

use super::OperationalId;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalStatusRequest {
    General,
}
