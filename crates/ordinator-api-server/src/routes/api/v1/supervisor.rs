use std::sync::Arc;
use std::sync::Mutex;

use axum::Router;
use axum::routing::get;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;

use crate::handlers::supervisor_handlers::status;

pub async fn supervisor_routes(state: Arc<Orchestrator<TotalSystemSolution>>) -> Router
{
    Router::new()
        .route("/:asset/:supervisor_id", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
