use axum::Router;

// Making a `status` for each actor is probably a really good idea.
pub async fn tactical_route() -> Router
{
    Router::new().route("/", status)
}

// TODO [ ]
// let orchestrator = orchestrator.lock().unwrap();
// Ok(orchestrator
//     .handle_tactical_request(tactical_request)
//     .await?)
