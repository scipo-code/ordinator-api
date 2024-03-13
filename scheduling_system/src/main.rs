mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use agents::orchestrator::Orchestrator;

use crate::init::logging;

#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    let _guard = logging::setup_logging();

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52, 56),
    ));

    let orchestrator = Arc::new(Mutex::new(Orchestrator::new(
        scheduling_environment.clone(),
    )));

    HttpServer::new(move || {
        let orchestrator = orchestrator.clone();
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
