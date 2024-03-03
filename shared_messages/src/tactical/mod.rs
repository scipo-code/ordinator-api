use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TacticalRequest {
    Status,
    Scheduling,
    Resources,
    Days,
}

impl Message for TacticalRequest {
    type Result = ();
}
