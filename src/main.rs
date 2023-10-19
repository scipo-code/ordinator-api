mod agents;
mod models;
mod data_processing;
mod messages;
mod api;

use calamine::{Xlsx};
use std::io::BufReader;
use std::fs::File;
use crate::models::scheduling_environment::SchedulingEnvironment;
use std::collections::{HashMap, HashSet};


use actix::prelude::*;
use std::path::Path;
use std::env;
use crate::data_processing::sources::excel::load_data_file;



use actix_web::{HttpServer, App};
use crate::api::routes::ws_index;
use crate::agents::scheduler_agent::SchedulerAgent;


#[actix_web::main]
async fn main() -> std::io::Result<()> {






    let args: Vec<String> = env::args().collect();
    let xlsx: Xlsx<BufReader<File>>;
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = load_data_file(file_path).expect("Could not load data file.");
        println!("{}", scheduling_environment);
    } 

    let scheduler_agent = SchedulerAgent::new(
        String::from("Dan F"),
        HashMap::new(),
        Vec::new(),
        HashMap::new(),
        Vec::new()).start();

        
    // scheduler_agent.schedule().await;

    // let scheduling_environment = SchedulingEnvironment::initialize_from_sources(work_orders, worker_environment);

    HttpServer::new(|| {
        App::new().service(ws_index)
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}
