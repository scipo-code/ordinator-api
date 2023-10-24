use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;
use actix::prelude::*;
use std::sync::Arc;
use std::thread;
use tracing::{Level, info, event, instrument};


use crate::api::websocket_agent::WebSocketAgent;
use crate::agents::scheduler_agent::SchedulerAgent;



#[get("/ws")]
async fn ws_index(
    data: web::Data<Arc<Addr<SchedulerAgent>>>, 
    req: HttpRequest, 
    stream: web::Payload
) -> Result<HttpResponse> {

    let current_thread_id = thread::current().id();
    event!(Level::INFO, ?current_thread_id, "starting app");

    let res = ws::start(WebSocketAgent::new(data.get_ref().clone()), &req, stream);
    res
}


#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}