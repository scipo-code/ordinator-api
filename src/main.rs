mod agents;
mod models;
mod data_processing;
mod api;
mod messages;

use actix::prelude::*;
use std::path::Path;
use std::env;
use std::sync::Arc;
use std::collections::{HashMap};
use std::thread;
use actix_web::{web, App, HttpServer};
use actix::Addr;
use tracing::{Level, info, event, instrument};

use crate::models::scheduling_environment::SchedulingEnvironment;
use crate::data_processing::sources::excel::load_data_file;
use crate::api::routes::ws_index;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::agents::scheduler_agent::PriorityQueues;
use crate::agents::scheduler_agent::SchedulerAgentAlgorithm;
use crate::models::scheduling_environment::WorkOrders;

#[instrument]
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // set the max level for logging
        .init();

    let mut scheduling_environment = initialize_scheduling_environment().unwrap();
    scheduling_environment.work_orders.initialize_work_orders();

    println!("{}", scheduling_environment.work_orders);

    let cloned_work_orders = scheduling_environment.work_orders.clone();

    let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
        HashMap::new(),
        HashMap::new(),
        cloned_work_orders,
        PriorityQueues::new(),
        HashMap::new(),
        Vec::new(),
    );

    let scheduler_agent = SchedulerAgent::new(
        String::from("Dan F"),
        scheduler_agent_algorithm,  
        None);

    println!("{}", scheduler_agent);

    let scheduler_agent_addr: Arc<Addr<SchedulerAgent>> = Arc::new(scheduler_agent.start());

    // scheduler_agent_addr.send(SchedulerMessages::ExecuteIteration).await.unwrap();
    
    info!("Server running at http://127.0.0.1:8001/");
    HttpServer::new(move || {

        let current_thread_id = thread::current().id();
        event!(Level::INFO, ?current_thread_id, "starting app");

        App::new()
            .app_data(web::Data::new(scheduler_agent_addr.clone()))
            .service(ws_index)
    })
    .bind(("127.0.0.1", 8001))?
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