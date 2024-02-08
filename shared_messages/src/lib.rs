pub mod resources;
pub mod strategic;

use actix::dev::MessageResponse;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::strategic::StrategicRequests;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "message_type")]
pub enum FrontendMessages {
    Strategic(StrategicRequests),
    Tactical,
    Worker,
}

#[derive(Debug)]
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

impl<A, M> MessageResponse<A, M> for Response
where
    A: Actor,
    M: Message<Result = Response>,
{
    fn handle(
        self,
        ctx: &mut <A as Actor>::Context,
        msg: std::option::Option<actix::dev::OneshotSender<Response>>,
    ) {
    }
}
