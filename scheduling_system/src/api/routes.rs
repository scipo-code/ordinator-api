use actix::prelude::*;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;
use std::sync::Arc;
use std::thread;
use tracing::info;

use crate::agents::scheduler_agent::StrategicAgent;
use crate::api::websocket_agent::WebSocketAgent;

#[get("/ws")]
async fn ws_index(
    sche_actor_addr: web::Data<Arc<Addr<StrategicAgent>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse> {
    let current_thread_id = thread::current().id();
    info!(?current_thread_id, "Setting up ws_index route handler");

    let res = ws::start(
        WebSocketAgent::new(sche_actor_addr.get_ref().clone()),
        &req,
        stream,
    );
    res
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}
