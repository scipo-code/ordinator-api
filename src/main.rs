mod agents;
mod api;
mod data_processing;
mod init;
mod models;

// #[macro_use]
// extern crate lazy_static;
// use lazy_static::lazy_static;
// use jlrs::prelude::*;
use tracing::instrument;
use tracing::{event, Level};

use crate::init::application_builder::ApplicationBuilder;
use crate::init::logging::setup_logging;

#[instrument]
#[actix_web::main]
async fn main() -> () {
    let _guard = setup_logging();

    // let julia = unsafe {
    //     RuntimeBuilder::new()
    //         .async_runtime::<Tokio>()
    //         .start_async::<3>()
    //         .unwrap()
    // };
    // lazy_static! {
    //     pub static ref JULIA: Mutex<Julia> = Mutex::new(unsafe { Julia::init().unwrap() });
    // }

    event!(Level::INFO, "starting application");
    let scheduling_environment = init::model_initializers::create_scheduling_environment(10);

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
