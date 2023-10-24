mod agents;
mod models;
mod data_processing;
mod api;

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