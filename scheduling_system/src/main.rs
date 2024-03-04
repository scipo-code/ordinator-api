mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::{
    io,
    sync::{Arc, Mutex},
};

use actix::Actor;
use actix_web::{web, App, HttpServer};
use agents::orchestrator_agent::OrchestratorAgent;

use crate::init::logging;

#[actix_web::main]
async fn main() -> Result<(), MainError> {
    let _guard = logging::setup_logging();

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52),
    ));
    dbg!();

    let orchestrator_agent_addr = OrchestratorAgent::new(scheduling_environment.clone()).start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(orchestrator_agent_addr.clone()))
            .route(
                "/ws",
                web::post().to(api::routes::http_to_scheduling_system),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await;
    Ok(())
}

#[derive(Debug)]
struct MainError;

impl From<io::Error> for MainError {
    fn from(err: io::Error) -> Self {
        println!("IO Error: {:?}", err);
        MainError
    }
}
