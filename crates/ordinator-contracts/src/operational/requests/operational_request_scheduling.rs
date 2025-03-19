use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalSchedulingRequest {
    OperationalIds,
    OperationalState(String),
}
