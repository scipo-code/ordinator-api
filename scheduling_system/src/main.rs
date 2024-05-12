mod agents;
mod api;
mod data_processing;
mod init;

use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use agents::orchestrator::Orchestrator;
use mongodb::{
    bson::{doc, Document},
    Client,
};
use mongodb::{options::ClientOptions, results::InsertOneResult};
use shared_messages::{models::SchedulingEnvironment, Asset};
use tracing::info;

use crate::init::logging;

///This is the entry point of the application
#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    dotenv::dotenv().ok();

    let log_handles = logging::setup_logging();

    let ordinator_database = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();

    let scheduling_environment = match ordinator_database
        .database("ordinator")
        .collection("scheduling_environment")
        .find_one(None, None)
        .await
        .unwrap()
    {
        Some(scheduling_environment) => {
            info!("SchedulingEnvironment loaded from mongodb");
            scheduling_environment
        }
        None => {
            let scheduling_environment =
                init::model_initializers::initialize_scheduling_environment(52, 4, 120);
            let collection = ordinator_database
                .database("ordinator")
                .collection::<SchedulingEnvironment>("scheduling_environment");
            collection
                .insert_one(&scheduling_environment, None)
                .await
                .unwrap();
            info!("SchedulingEnvironment created from excel data");
            scheduling_environment
        }
    };

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
