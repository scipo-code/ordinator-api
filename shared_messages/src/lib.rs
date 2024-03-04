pub mod resources;
pub mod status;
pub mod strategic;
pub mod tactical;

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use status::StatusRequest;

use crate::strategic::StrategicRequest;
use crate::tactical::TacticalRequest;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "message_type")]
pub enum SystemMessages {
    Status(StatusRequest),
    Strategic(StrategicRequest),
    Tactical(TacticalRequest),
    Operational,
}

impl Message for SystemMessages {
    type Result = Response;
}

#[derive(Debug, Clone)]
pub enum Response {
    Success(Option<String>),
    Failure,
}

impl Message for Response {
    type Result = ();
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Response::Success(string) => match string {
                Some(string) => string.clone(),
                None => "Command was successfully received and integrated".to_string(),
            },
            Response::Failure => {
                "Command was failed to be either received or integrated".to_string()
            }
        }
    }
}


