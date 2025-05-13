use std::sync::Arc;
use std::sync::Mutex;

use axum::Router;
use axum::extract::Path;
use axum::routing::get;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;

use crate::handlers::scheduler_excel_export;
use crate::handlers::strategic_handlers::get_scheduler_work_orders;

// TODO [x]
// The main idea is to replace all the.
pub async fn scheduler_nest(
    state: Arc<Mutex<Orchestrator<TotalSystemSolution>>>,
) -> Router<Arc<Mutex<Orchestrator<TotalSystemSolution>>>>
{
    Router::new()
        .route("/work_orders/:id", get(get_scheduler_work_orders))
        .with_state(state)
}

// let asset = strategic_request.asset;
// let orchestrator_guard = orchestrator.lock().unwrap();

// let strategic = &orchestrator_guard
//     .agent_registries
//     .get(&asset)
//     .unwrap()
//     .strategic_agent_sender;

// strategic
//     .sender
//     .send(crate::agents::ActorMessage::Actor(
//         strategic_request.strategic_request_message,
//     ))
//     .map_err(actix_web::error::ErrorInternalServerError)?;

// let response = strategic
//     .receiver
//     .recv()
//     .map_err(actix_web::error::ErrorInternalServerError)?;
// drop(orchestrator_guard);

// let strategic_response_message = match response {
//     Ok(message) => message,
//     Err(e) => {
//         let error = format!("{:?}", e.context("http request could not be
// completed"));         return Ok(HttpResponse::BadRequest().body(error));
//     }
// };

// let strategic_response = StrategicResponse::new(asset,
// strategic_response_message);

// let system_message = SystemResponses::Strategic(strategic_response);

// Ok(HttpResponse::Ok().json(system_message))
