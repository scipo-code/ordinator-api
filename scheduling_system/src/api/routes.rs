use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_messages::SystemMessages;

use crate::agents::orchestrator_agent::OrchestratorAgent;

pub async fn http_to_scheduling_system(
    orchestrator_agent_addr: web::Data<Addr<OrchestratorAgent>>,
    req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> Result<HttpResponse> {
    dbg!(payload.0.clone());
    let response = orchestrator_agent_addr.send(payload.0).await;

    Ok(HttpResponse::Ok().json(response.unwrap()))
}
