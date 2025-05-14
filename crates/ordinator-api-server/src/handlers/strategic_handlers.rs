use std::sync::Arc;
use std::sync::Mutex;

use axum::extract::Path;
use axum::extract::State;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;
use ordinator_orchestrator::WorkOrderNumber;

// This is a handler. Not a `Route` you should change that. Keep working.
pub async fn get_scheduler_work_orders(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(i): Path<u64>,
)
{
    // This should go into the handler, directory. There is no other way around it
    // REMEMBER: You should only wrap method calls that the Orchestrator exposes.
    //
    // WARN: You are beginning to feel drained again. You should grap something to
    // eat again.
    let work_order_number = WorkOrderNumber(i);
    // TODO [ ] M
    // orchestrator.get_work_order(id)
}
