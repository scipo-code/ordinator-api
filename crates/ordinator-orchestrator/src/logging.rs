use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;

use tracing::Level;
use tracing::event;
use tracing_appender::non_blocking::NonBlocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_flame::FlameLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::fmt::format::Json;
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload;
use tracing_subscriber::reload::Handle;

type LogLayer = Handle<
    Filtered<Layer<Registry, JsonFields, Format<Json>, NonBlocking>, EnvFilter, Registry>,
    Registry,
>;
type ProfilingLayer = Filtered<FlameLayer<Registry, BufWriter<File>>, EnvFilter, Registry>;

#[derive(Debug)]
pub struct LogHandles
{
    pub file_handle: LogLayer,
    pub _flame_handle: Handle<ProfilingLayer, Registry>,
    pub _guard: WorkerGuard,
}

// TODO [ ]
// I think that this should be removed and replaced by the
// `tracing` crate. Yes you should
// #[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
// pub enum LogLevel {
//     Trace,
//     Debug,
//     Info,
//     Warn,
//     Error,
// }

// impl LogLevel {
//     pub fn to_level_string(&self) -> String {
//         match self {
//             LogLevel::Trace => "trace".to_string(),
//             LogLevel::Debug => "debug".to_string(),
//             LogLevel::Info => "info".to_string(),
//             LogLevel::Warn => "warn".to_string(),
//             LogLevel::Error => "error".to_string(),
//         }
//     }
// }

pub fn setup_logging() -> LogHandles
{
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
    let file_name = "ordinator.developer.log".to_string();

    let file_appender = tracing_appender::rolling::never(log_dir, file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .json()
        .with_ansi(true)
        .with_file(true) // Include file name in logs
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true) // Include line number in logs
        .with_filter(EnvFilter::from_env("TRACING_LEVEL"));

    let (file_layer, file_handle) = reload::Layer::new(file_layer);

    let flame_layer = FlameLayer::with_file(
        env::var("PROFILING_FILE").expect("A file name for the profiling data has to be set"),
    )
    .unwrap()
    .0
    .with_filter(EnvFilter::from_env("PROFILING_LEVEL"));
    let (flame_layer, _flame_handle) = reload::Layer::new(flame_layer);

    let layers = vec![file_layer.boxed(), flame_layer.boxed()];

    // So the `schedule()` function works correctly. But where is the bug
    // introduced? I really have to find this as the next step. I do not see a
    // different way of going about it.
    tracing_subscriber::registry().with(layers).init();

    event!(Level::INFO, "starting loging");
    LogHandles {
        file_handle,
        _flame_handle,
        _guard,
    }
}
