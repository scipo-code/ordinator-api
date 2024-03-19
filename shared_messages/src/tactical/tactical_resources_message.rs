use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceMessage {
    SetResources,
    GetLoadings,
    GetCapacities,
}
