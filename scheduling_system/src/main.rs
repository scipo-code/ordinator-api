mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::sync::{Arc, Mutex};

use crate::init::logging;
use crate::init::ordinator_builder::OrdinatorBuilder;

#[actix_web::main]
async fn main() -> () {
    let _guard = logging::setup_logging();

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52),
    ));

    let agent_factory = init::agent_factory::AgentFactory::new(Arc::clone(&scheduling_environment));

    let application_builder = OrdinatorBuilder::new(scheduling_environment, agent_factory)
        .build()
        .await;

    application_builder.await.unwrap();
}

#[cfg(test)]
mod tests {}
