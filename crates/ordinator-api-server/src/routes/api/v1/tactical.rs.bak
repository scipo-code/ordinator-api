use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;

use crate::handlers::tactical_handlers::status;

// Making a `status` for each actor is probably a really good idea.
pub async fn tactical_route(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> Router<Arc<Orchestrator<TotalSystemSolution>>>
{
    Router::new()
        .route("/", get(status::<TotalSystemSolution>))
        .with_state(state)
}

// TODO [ ]
// let orchestrator = orchestrator.lock().unwrap();
// Ok(orchestrator
//     .handle_tactical_request(tactical_request)
//     .await?)
