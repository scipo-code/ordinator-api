use actix::prelude::*;
use actix_web_actors::ws;
use crate::messages::scheduler_message::{RawInputMessage, InputMessage};
use actix_web::{HttpRequest, HttpResponse, Result};

pub struct MessageAgent;

impl Actor for MessageAgent {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageAgent {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {

                let raw_input_scheduler_data: Result<RawInputMessage, serde_json::Error> = serde_json::from_str(&text);
                let input_scheduler_data: InputMessage;
                match raw_input_scheduler_data {
                    Ok(raw_input_scheduler_data) => {
                        input_scheduler_data = raw_input_scheduler_data.into();
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        return;
                    }
                }

                println!("{}", input_scheduler_data);
          
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}