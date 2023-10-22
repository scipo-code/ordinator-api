use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;
use actix::prelude::*;
use std::sync::Arc;
use crate::api::websocket::MessageAgent;
use crate::agents::scheduler_agent::SchedulerAgent;

#[get("/")]
async fn ws_index(
    data: web::Data<Arc<Addr<SchedulerAgent>>>, 
    req: HttpRequest, 
    stream: web::Payload
) -> Result<HttpResponse> {
    let res = ws::start(MessageAgent::new(data.get_ref().clone()), &req, stream);
    res
}


#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}