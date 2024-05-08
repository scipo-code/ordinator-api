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
use shared_messages::{models::SchedulingEnvironment, Asset};
use surrealdb::{engine::remote::ws::Ws, opt::IntoResource, Response, Surreal};

use crate::init::logging;

///This is the entry point of the application
#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    dotenv::dotenv().ok();

    let log_handles = logging::setup_logging();

    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();

    let scheduling_environment =
        init::model_initializers::initialize_scheduling_environment(52, 4, 120);

    db.create::<Vec<SchedulingEnvironment>>("SchedulingEnvironment")
        .content(serde_json::to_string(&scheduling_environment).unwrap())
        .await;

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
