use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalSchedulingRequest {
    OperationalIds,
    OperationalState(String),
}
