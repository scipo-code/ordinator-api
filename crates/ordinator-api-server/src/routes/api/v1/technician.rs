use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;

use crate::handlers::orchestrator_handlers::orchestrator_requests;

// This function is only for providing the correct routes.
pub async fn call_orchestrator(state: Arc<Orchestrator<TotalSystemSolution>>) -> Router
{
    Router::new()
        .route("/", get(orchestrator_requests))
        .with_state(state)
}
