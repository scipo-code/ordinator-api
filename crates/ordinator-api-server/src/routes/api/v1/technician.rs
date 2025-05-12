use axum::Router;
use axum::routing::Route;
use axum::routing::get;
use ordinator_orchestrator::OrchestratorRequest;

use crate::handlers::orchestrator_handlers::orchestrator_requests;

// This function is only for providing the correct routes.
pub async fn call_orchestrator() -> Router
{
    Router::new().route("/", get(orchestrator_requests))
}
