mod agents;
mod models;
mod data_processing;
<<<<<<< HEAD
mod api;

=======
mod messages;

use calamine::{Xlsx};
use std::io::BufReader;
use std::fs::File;
use crate::models::scheduling_environment::SchedulingEnvironment;

use actix::prelude::*;
use actix_web_actors::ws;
use std::path::Path;
>>>>>>> origin
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::collections::{HashMap};
use std::thread;
use actix::prelude::*;

use actix_web::{web, App, HttpServer};
use actix::Addr;



use tracing::{Level, info, event, instrument};

use crate::data_processing::sources::excel::load_data_file;
use crate::api::routes::ws_index;
use crate::agents::scheduler_agent::SchedulerAgent;

<<<<<<< HEAD
use crate::models::scheduling_environment::SchedulingEnvironment;

#[instrument]
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // set the max level for logging
        .init();

    let mut scheduling_environment = initialize_scheduling_environment();

    for (_, work_order) in scheduling_environment.as_mut().unwrap().work_orders.inner.iter_mut() {
        work_order.calculate_weight();
    }

    let cloned_work_orders = scheduling_environment.as_ref().unwrap().get_work_orders().clone();

    println!("{}", cloned_work_orders);

    let scheduler_agent: Arc<Addr<SchedulerAgent>> = Arc::new(SchedulerAgent::new(
        String::from("Dan F"),
        HashMap::new(),
        cloned_work_orders,
        HashMap::new(),
        Vec::new(),
        None,
        ).start());

    // scheduler_agent.send(SchedulerMessages::ExecuteIteration).await.unwrap();
    
    info!("Server running at http://127.0.0.1:8001/");
    HttpServer::new(move || {

        let current_thread_id = thread::current().id();
        event!(Level::INFO, ?current_thread_id, "starting app");

        App::new()
            .app_data(web::Data::new(scheduler_agent.clone()))
            .service(ws_index)
    })
    .bind(("127.0.0.1", 8001))?
=======
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

>>>>>>> origin
    .run()
    .await
}

fn initialize_scheduling_environment() -> Option<SchedulingEnvironment> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = load_data_file(file_path).expect("Could not load data file.");
        println!("{}", scheduling_environment);
        return Some(scheduling_environment);
    } 
    None
}