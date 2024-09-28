use std::fs::File;
use std::io::BufWriter;
use std::{env, fs};
use tracing::{event, Level};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_flame::FlameLayer;
use tracing_subscriber::filter::{EnvFilter, Filtered};
use tracing_subscriber::fmt::format::{Format, Json, JsonFields};
use tracing_subscriber::fmt::{self, Layer};
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{prelude::*, reload, Registry};

type LogLayer =
    Filtered<Layer<Registry, JsonFields, Format<Json>, NonBlocking>, EnvFilter, Registry>;
type ProfilingLayer = Filtered<FlameLayer<Registry, BufWriter<File>>, EnvFilter, Registry>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LogHandles {
    pub file_handle: Handle<LogLayer, Registry>,
    pub flame_handle: Handle<ProfilingLayer, Registry>,
}

pub fn setup_logging() -> (LogHandles, WorkerGuard) {
    let previous_log_files = fs::read_dir(
        dotenvy::var("ORDINATOR_LOG_DIR")
            .expect("The ORDINATOR_LOG_DIR environment variables should always be set."),
    )
    .unwrap();

    for log_file in previous_log_files {
        let path = log_file.unwrap().path();
        if path.file_name().unwrap() == ".gitkeep" {
            continue;
        };
        if path.is_file()
            && path
                .extension()
                .expect("All files in the logs directory should have the .log file extension")
                == "log"
        {
            fs::remove_file(path).expect("If you encounter this error ");
        }
    }

    let log_dir = env::var("ORDINATOR_LOG_DIR")
        .expect("A logging/tracing directory should be set in the .env file");
    let file_name = format!("ordinator.developer.log");

    let file_appender = tracing_appender::rolling::never(log_dir, file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .json() // Output logs in JSON format
        .with_file(true) // Include file name in logs
        .with_thread_ids(true)
        .with_line_number(true) // Include line number in logs
        .with_current_span(true)
        .with_filter(EnvFilter::from_env("TRACING_LEVEL"));

    let (file_layer, file_handle) = reload::Layer::new(file_layer);

    let flame_layer = FlameLayer::with_file("PROFILING_FILE")
        .unwrap()
        .0
        .with_filter(EnvFilter::from_env("PROFILING_LEVEL"));
    let (flame_layer, flame_handle) = reload::Layer::new(flame_layer);

    let layers = vec![file_layer.boxed(), flame_layer.boxed()];

    tracing_subscriber::registry().with(layers).init();

    event!(Level::INFO, "starting loging");
    let log_handles = LogHandles {
        file_handle,
        flame_handle,
    };
    (log_handles, _guard)
}
