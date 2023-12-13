use actix::Message;
use std::env;
use tracing::{event, Level};
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;

trait AgentLogger {
    fn log_message_received(&self, message: dyn Message<Result = ()>);
}

pub fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = env::var("LOG_DIR").unwrap_or("./logs".to_string());
    let file_appender = tracing_appender::rolling::daily(log_dir, "ordinator.log");
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
