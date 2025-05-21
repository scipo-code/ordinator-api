use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;

use crate::handlers::orchestrator_handlers::get_days;
use crate::handlers::orchestrator_handlers::orchestrator_status;
use crate::handlers::orchestrator_handlers::scheduler_excel_export;

pub async fn export_xlsx(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> Router<Arc<Orchestrator<TotalSystemSolution>>>
{
    Router::new()
        .route("/export_xlsx/{asset}", get(scheduler_excel_export))
        .with_state(state)
}

// This function is only for providing the correct routes.
pub async fn orchestrator_api_scope(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> Router<Arc<Orchestrator<TotalSystemSolution>>>
{
    Router::new()
        .route("/", get(orchestrator_status))
        .route("/number_of_days", get(get_days))
        .with_state(state)
}
