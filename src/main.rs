mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::sync::{Arc, Mutex};

use tracing::{info, instrument};

use crate::init::application_builder::ApplicationBuilder;
use crate::init::logging::setup_logging;

#[instrument]
#[actix_web::main]
async fn main() -> () {
    let _guard = setup_logging();

    info!("starting application");
    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::create_scheduling_environment(10),
    ));
    let scheduler_agent_addr = init::agent_factory::build_scheduler_agent(scheduling_environment);

    ApplicationBuilder::new()
        .with_scheduler_agent(scheduler_agent_addr)
        // put other agents here
        .build()
        .await
        .expect("could not start application");
}

#[cfg(test)]
mod tests {}
