use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use file_rotate::ContentLimit;
use file_rotate::FileRotate;
use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use tracing::Level;
use tracing::event;
use tracing_appender::non_blocking::NonBlocking;
use tracing_flame::FlameLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::filter;
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
    let research_log_file: PathBuf = (log_dir.clone() + "ordinator.developer.log").into();
    let operational_logging_path: PathBuf = (log_dir.clone() + "ordinator.developer.log").into();

    let research_file = FileRotate::new(
        research_log_file,
        AppendCount::new(1),
        ContentLimit::Bytes(1024 * 1024 * 1024),
        Compression::None,
        None,
    );

    let (research_non_blocking, research_log_guard) = tracing_appender::non_blocking(research_file);
    std::mem::forget(research_log_guard);

    let research_layer = fmt::layer()
        .with_writer(research_non_blocking)
        .json()
        .with_ansi(true)
        .with_file(true) // Include file name in logs
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true) // Include line number in logs
        .with_filter(EnvFilter::from_env("TRACING_LEVEL"));

    let (research_layer, file_handle) = reload::Layer::new(research_layer);

    let developer_file = FileRotate::new(
        operational_logging_path,
        AppendCount::new(1),
        ContentLimit::Bytes(50 * 1024 * 1024),
        Compression::None,
        None,
    );

    let (operational_nb, developer_log_guard) = tracing_appender::non_blocking(developer_file);
    std::mem::forget(developer_log_guard);

    let developer_layer = fmt::layer()
        .with_writer(operational_nb)
        .json()
        .with_ansi(true)
        .with_file(true) // Include file name in logs
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true) // Include line number in logs
        .with_filter(filter::LevelFilter::DEBUG);

    let flame_layer = FlameLayer::with_file(
        env::var("PROFILING_FILE").expect("A file name for the profiling data has to be set"),
    )
    .unwrap()
    .0
    .with_filter(EnvFilter::from_env("PROFILING_LEVEL"));
    let (flame_layer, _flame_handle) = reload::Layer::new(flame_layer);

    let layers = vec![
        research_layer.boxed(),
        flame_layer.boxed(),
        developer_layer.boxed(),
    ];

    // So the `schedule()` function works correctly. But where is the bug
    // introduced? I really have to find this as the next step. I do not see a
    // different way of going about it.
    tracing_subscriber::registry().with(layers).init();

    event!(Level::INFO, "starting loging");
    LogHandles {
        file_handle,
        _flame_handle,
    }
}
