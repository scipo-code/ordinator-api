use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;

use crate::api::websocket::MessageAgent;

#[get("/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    dbg!();
    let res = ws::start(MessageAgent {}, &req, stream);
    dbg!();
    println!("{:?}", res.as_ref().unwrap().body());
    res
}


#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}