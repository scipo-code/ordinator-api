use axum::Router;

pub async fn supervisor_routes() -> Router
{
    Router::new().route("/", method_router)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
