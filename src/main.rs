mod agents;
mod models;
mod data_processing;
mod api;
mod init;

use tracing::instrument;
use crate::init::logging::setup_logging;
use tracing::{event, Level};

use crate::init::application_builder::ApplicationBuilder;

#[instrument]
#[actix_web::main]
async fn main() -> () {
    let _guard = setup_logging();

    event!(Level::INFO, "starting application");
    let scheduling_environment = init::model_initializers::create_scheduling_environment(10);

    let scheduler_agent_addr = init::agent_factory::build_scheduler_agent(scheduling_environment);

    ApplicationBuilder::new()
        .with_scheduler_agent(scheduler_agent_addr)
        // put other agents here
        .build()
        .await.expect("could not start application");
}

