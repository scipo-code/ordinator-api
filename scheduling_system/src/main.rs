mod agents;
mod api;
mod data_processing;
mod init;

use futures_util::io::AsyncWriteExt;
use std::{
    fs::File,
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use agents::orchestrator::Orchestrator;
use mongodb::{bson::doc, options::GridFsBucketOptions, Client};

use shared_messages::{models::SchedulingEnvironment, Asset};
use tracing::info;

use crate::init::logging;

///This is the entry point of the application. We should
#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    dotenv::dotenv().ok();

    let log_handles = logging::setup_logging();

    let scheduling_environment: SchedulingEnvironment;
    if std::path::Path::new("temp_scheduling_environment/scheduling_environment.json").exists() {
        let mut file =
            File::open("temp_scheduling_environment/scheduling_environment.json").unwrap();
        let mut data = String::new();

        file.read_to_string(&mut data).unwrap();

        scheduling_environment = serde_json::from_str(&data).unwrap();
    } else {
        scheduling_environment =
            init::model_initializers::initialize_scheduling_environment(52, 4, 120);

        let json_scheduling_environment = serde_json::to_string(&scheduling_environment).unwrap();
        let mut file =
            File::create("temp_scheduling_environment/scheduling_environment.json").unwrap();

        file.write_all(json_scheduling_environment.as_bytes())
            .unwrap();
    }

    let mutex_scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

    let mut orchestrator = Orchestrator::new(mutex_scheduling_environment.clone(), log_handles);

    orchestrator.add_asset(Asset::DF);
    // orchestrator.add_asset(Asset::HD);
    let arc_orchestrator = Arc::new(Mutex::new(orchestrator));

    HttpServer::new(move || {
        let orchestrator = arc_orchestrator.clone();
        App::new().app_data(web::Data::new(orchestrator)).route(
            "/ws",
            web::post()
                .guard(guard::Header("content-type", "application/json"))
                .to(api::routes::http_to_scheduling_system),
        )
    })
    .workers(4)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
