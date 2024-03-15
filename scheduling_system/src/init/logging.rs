use std::env;
use tracing::{event, Level};
use tracing_appender::non_blocking::NonBlocking;
use tracing_flame::FlameLayer;
use tracing_subscriber::filter::{EnvFilter, Filtered};
use tracing_subscriber::fmt::format::{Format, Json, JsonFields};
use tracing_subscriber::fmt::{self, Layer};
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{prelude::*, reload, Registry};

pub struct LogHandles {
    pub file_handle: Handle<
        Filtered<Layer<Registry, JsonFields, Format<Json>, NonBlocking>, EnvFilter, Registry>,
        Registry,
    >,
    guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn setup_logging() -> LogHandles {
    let log_dir = env::var("LOG_DIR").unwrap_or("./logs".to_string());
    let file_appender = tracing_appender::rolling::daily(log_dir, "ordinator.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .json() // Output logs in JSON format
        .with_file(true) // Include file name in logs
        .with_line_number(true) // Include line number in logs
        .with_filter(EnvFilter::from_default_env());
    let (file_layer, file_handle) = reload::Layer::new(file_layer);

    let (flame_layer, _guard) =
        FlameLayer::with_file("./profiling_and_benchmarking/tracing.folded").unwrap();
    let (flame_layer, flame_handle) = reload::Layer::new(flame_layer);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(flame_layer)
        .with(EnvFilter::from_default_env())
        .init();

    file_handle.reload(EnvFilter::new("debug")).unwrap();
    event!(Level::INFO, "starting loging");
    LogHandles { file_handle, guard }
}
