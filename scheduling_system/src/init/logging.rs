use std::env;
use std::fs::File;
use std::io::BufWriter;
use tracing::{event, Level};
use tracing_appender::non_blocking::NonBlocking;
use tracing_flame::FlameLayer;
use tracing_subscriber::filter::{EnvFilter, Filtered};
use tracing_subscriber::fmt::format::{Format, Json, JsonFields};
use tracing_subscriber::fmt::{self, Layer};
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{prelude::*, reload, Registry};

type LogLayer =
    Filtered<Layer<Registry, JsonFields, Format<Json>, NonBlocking>, EnvFilter, Registry>;
type ProfilingLayer = Filtered<FlameLayer<Registry, BufWriter<File>>, EnvFilter, Registry>;
pub struct LogHandles {
    pub file_handle: Handle<LogLayer, Registry>,
    pub flame_handle: Handle<ProfilingLayer, Registry>,
    _guard: tracing_appender::non_blocking::WorkerGuard,
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

    let flame_layer = FlameLayer::with_file("./profiling_and_benchmarking/tracing.folded")
        .unwrap()
        .0
        .with_filter(EnvFilter::from_default_env());
    let (flame_layer, flame_handle) = reload::Layer::new(flame_layer);

    let layers = vec![file_layer.boxed(), flame_layer.boxed()];

    tracing_subscriber::registry().with(layers).init();

    event!(Level::INFO, "starting loging");
    LogHandles {
        file_handle,
        flame_handle,
        _guard: guard,
    }
}
