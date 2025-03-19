use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum TacticalTimeRequest {
    Days,
}
