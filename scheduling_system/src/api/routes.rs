use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::info;

use crate::agents::orchestrator_agent::OrchestratorAgent;
use crate::models::SchedulingEnvironment;

#[get("/ws")]
async fn ws_index(
    scheduling_environment: web::Data<Arc<Mutex<SchedulingEnvironment>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse> {
    let current_thread_id = thread::current().id();
    info!(?current_thread_id, "Setting up ws_index route handler");

    let res = ws::start(
        OrchestratorAgent::new(scheduling_environment.get_ref().clone()),
        &req,
        stream,
    );
    res
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}
