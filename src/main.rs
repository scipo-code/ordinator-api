mod agents;
mod models;
mod data_processing;
mod messages;

use calamine::{Xlsx};
use std::io::BufReader;
use std::fs::File;
use crate::models::scheduling_environment::SchedulingEnvironment;

use actix::prelude::*;
use actix_web_actors::ws;
use std::path::Path;
use std::env;
use crate::data_processing::sources::excel::load_data_file;

use crate::messages::scheduler_message::{RawInputMessage, InputMessage};

use actix_web::{get, web, App, HttpServer, HttpRequest, HttpResponse, Result};

struct MessageAgent;

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

#[get("/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    dbg!();
    let res = ws::start(MessageAgent {}, &req, stream);
    dbg!();
    println!("{:?}", res.as_ref().unwrap().body());
    res
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let xlsx: Xlsx<BufReader<File>>;
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = load_data_file(file_path).expect("Could not load data file.");
        println!("{}", scheduling_environment);
    } 

    // let scheduling_environment = SchedulingEnvironment::initialize_from_sources(work_orders, worker_environment);

    HttpServer::new(|| {
        App::new().service(ws_index)
    })
    .bind(("127.0.0.1", 8001))?

    .run()
    .await
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}