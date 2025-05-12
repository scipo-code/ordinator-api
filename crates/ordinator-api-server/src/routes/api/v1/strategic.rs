use axum::Router;
use axum::extract::Path;
use axum::routing::get;

// TODO [x]
// The main idea is to replace all the.
pub async fn scheduler_nest() -> Router
{
    Router::new().route("/work_orders/:id", get(get_scheduler_work_orders))
}

pub async fn get_scheduler_work_orders(Some(Path(id)): Option<Path<u64>>)
{
    orchestrator.get_work_order(id)
}
