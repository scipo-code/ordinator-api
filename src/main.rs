mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use clap::Parser;
use models::SchedulingEnvironment;
use std::sync::{Arc, Mutex};
use tracing::{info, instrument};

use crate::init::application_builder::ApplicationBuilder;
use crate::init::logging::setup_logging;

#[actix_web::main]
async fn main() -> () {
    let _guard = setup_logging();

    info!("starting application");

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52),
    ));

    let scheduler_agent_addr = init::agent_factory::build_scheduler_agent(scheduling_environment);

    // command_line_interface();

    let application_builder = ApplicationBuilder::new()
        .with_scheduler_agent(scheduler_agent_addr)
        .build()
        .await;

    application_builder.await.unwrap();
}

#[cfg(test)]
mod tests {}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CommandLineInterface {
    get_number_of_agents: String,
    // number_of_periods: u32,
    // number_of_work_orders: u32,
    // system_agent_registry: SystemAgentRegistry,
    // scheduling_environment: Arc<Mutex<models::SchedulingEnvironment>>,
}

// async fn command_line_interface(system_agent_registry: SystemAgentRegistry) {
//     let cli = Cli::parse();

//     loop {

//     }
// }
