use std::sync::Arc;

use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;
use ordinator_orchestrator::WorkOrderNumber;

// This is a handler. Not a `Route` you should change that. Keep working.
#[debug_handler]
pub async fn get_scheduler_work_orders(
    State(_orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(i): Path<u64>,
) -> Result<Json<WorkOrderNumber>>
{
    // This should go into the handler, directory. There is no other way around it
    // REMEMBER: You should only wrap method calls that the Orchestrator exposes.
    //
    // WARN: You are beginning to feel drained again. You should grap something to
    // eat again.
    Ok(Json(WorkOrderNumber(i)))

    // TODO [ ] M
    // orchestrator.get_work_order(id)
}
