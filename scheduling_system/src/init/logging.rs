use actix::Message;
use std::env;
use tracing::{event, Level};
use tracing_flame::FlameLayer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;

trait AgentLogger {
    fn log_message_received(&self, message: dyn Message<Result = ()>);
}

pub fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = env::var("LOG_DIR").unwrap_or("./logs".to_string());
    let file_appender = tracing_appender::rolling::daily(log_dir, "ordinator.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let (flame_layer, _guard) =
        FlameLayer::with_file("./profiling_and_benchmarking/tracing.folded").unwrap();

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .json() // Output logs in JSON format
        .with_file(true) // Include file name in logs
        .with_line_number(true); // Include line number in logs

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true) // Include log target (e.g., module path) in logs
        .with_span_events(FmtSpan::CLOSE); // Log span closure events

    tracing_subscriber::registry()
        .with(flame_layer)
        .with(file_layer)
        .with(stdout_layer)
        .with(EnvFilter::from_default_env())
        .init();

    event!(Level::INFO, "starting loging");
    guard
}
