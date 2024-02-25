mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::sync::{Arc, Mutex};

use crate::init::application_builder::ApplicationBuilder;
use crate::init::logging;

#[actix_web::main]
async fn main() -> () {
    let _guard = logging::setup_logging();

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52),
    ));

    let scheduler_agent_addr =
        init::agent_factory::build_scheduler_agent(Arc::clone(&scheduling_environment));

    let tactical_agent_addr =
        init::agent_factory::build_tactical_agent(Arc::clone(&scheduling_environment));

    let application_builder = ApplicationBuilder::new()
        .with_scheduler_agent(scheduler_agent_addr)
        .with_tactical_agent(tactical_agent_addr)
        .build()
        .await;

    application_builder.await.unwrap();
}

#[cfg(test)]
mod tests {}
