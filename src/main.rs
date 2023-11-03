mod agents;
mod models;
mod data_processing;
mod api;

use actix::prelude::*;
use agents::scheduler_agent::OptimizedWorkOrders;
use std::path::Path;
use std::env;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::thread;
use actix_web::{web, App, HttpServer};
use actix::Addr;
use tracing::{Level, info, event, instrument};

use crate::models::scheduling_environment::WorkOrders;
use crate::models::scheduling_environment::SchedulingEnvironment;
use crate::data_processing::sources::excel::load_data_file;
use crate::data_processing::sources::excel_joins::read_csv_files;
use crate::agents::scheduler_agent::{SchedulerAgent, OptimizedWorkOrder};
use crate::agents::scheduler_agent::PriorityQueues;
use crate::agents::scheduler_agent::SchedulerAgentAlgorithm;
use crate::api::routes::ws_index;



#[instrument]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args[1] == "create-excel-data" {
        read_csv_files();
        return Ok(());
    }
    

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // set the max level for logging
        .init();

    let number_of_periods = 52;

    let mut scheduling_environment = initialize_scheduling_environment(number_of_periods).unwrap();
    scheduling_environment.work_orders.initialize_work_orders();

    println!("{}", scheduling_environment.work_orders);

    let cloned_work_orders = scheduling_environment.work_orders.clone();

    let optimized_work_orders: OptimizedWorkOrders = create_optimized_work_orders(&cloned_work_orders);
    // We really should create a initialize state function.

    let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
        HashMap::new(),
        HashMap::new(),
        cloned_work_orders,
        PriorityQueues::new(),
        optimized_work_orders,
        scheduling_environment.period.clone(),
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

fn initialize_scheduling_environment(number_of_periods: u32) -> Option<SchedulingEnvironment> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = load_data_file(file_path, number_of_periods).expect("Could not load data file.");
        println!("{}", scheduling_environment);
        return Some(scheduling_environment);
    } 
    None
}

fn create_optimized_work_orders(work_orders: &WorkOrders) -> OptimizedWorkOrders {
    
    let mut optimized_work_orders: HashMap<u32, OptimizedWorkOrder> = HashMap::new();

    for (work_order_number, work_order) in &work_orders.inner {
        if work_order.unloading_point.present {
            let period = work_order.unloading_point.period.clone();
            optimized_work_orders.insert(*work_order_number, OptimizedWorkOrder::new(
                period.clone(),
                period,
                HashSet::new(),
            ));
        }
    }
    OptimizedWorkOrders::new(optimized_work_orders)
}