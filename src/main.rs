mod agents;
mod models;
mod data_processing;
mod api;
mod init;

use tracing::instrument;
use tracing::{event, Level, span, Id, Subscriber, Event};
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_subscriber::prelude::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};



use crate::data_processing::create_excel_data;
use crate::init::application_builder::ApplicationBuilder;


#[instrument]
#[actix_web::main]
async fn main() -> () {
    create_excel_data();

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

fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {

    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    let subscriber = fmt::layer()
        .with_writer(non_blocking)
        .json() // Output logs in JSON format
        .with_file(true) // Include file name in logs
        .with_line_number(true); // Include line number in logs

        tracing_subscriber::registry().with(subscriber).init();

    event!(Level::INFO, "starting loging");
    event!(Level::ERROR, "starting loging");
    guard
}