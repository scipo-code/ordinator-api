mod agents;
mod models;
mod data_processing;
mod api;
mod init;

#[macro_use]
extern crate lazy_static;
use lazy_static::lazy_static;
use tracing::instrument;
use tracing::{event, Level};
use jlrs::prelude::*;

use crate::init::logging::setup_logging;
use crate::init::application_builder::ApplicationBuilder;

#[instrument]
#[actix_web::main]
async fn main() -> () {
    let _guard = setup_logging();

    let julia = unsafe {
        RuntimeBuilder::new()
            .async_runtime::<Tokio>()
            .start_async::<3>()
            .unwrap()
    };
    lazy_static! {
        pub static ref JULIA: Mutex<Julia> = Mutex::new(unsafe { Julia::init().unwrap() });
    }

    event!(Level::INFO, "starting application");
    let scheduling_environment = init::model_initializers::create_scheduling_environment(10);

    let scheduler_agent_addr = init::agent_factory::build_scheduler_agent(scheduling_environment);

    ApplicationBuilder::new()
        .with_scheduler_agent(scheduler_agent_addr)
        // put other agents here
        .build()
        .await.expect("could not start application");
}

