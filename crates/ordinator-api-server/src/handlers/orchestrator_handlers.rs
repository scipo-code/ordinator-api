use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::Result;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::OrchestratorRequest;
use ordinator_orchestrator::TotalSystemSolution;

// This should be deleted and replaced with the other handler. I do not
// see a different way around it.
// pub async fn handle_orchestrator_message<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     Json(reg): Json<OrchestratorRequest>,
// ) -> Result<Response, axum::Error>
// {
//     let mut orchestrator = orchestrator.lock().unwrap();

//     // Every handler should go into the... You are combining the
//     // handlers with the routes. You should not be doing that...
//     Ok(orchestrator.handle_orchestrator_request(reg).await?)
// }

pub async fn scheduler_excel_export(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<Asset>,
) -> Result<Response>
{
    let mut headers = HeaderMap::new();

    let (buffer, http_header) = orchestrator
        .export_xlsx_solution(asset)
        .expect("Could not export xlsx");

    headers.insert(
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            .parse()
            .unwrap(),
    );

    headers.insert(header::CONTENT_DISPOSITION, http_header.parse().unwrap());

    // let http_response = HttpResponse::Ok()
    //     .content_type("application/vnd.openxmlformats-officedocument.
    // spreadsheetml.sheet")     .insert_header(("Content-Disposition",
    // http_header))     .body(buffer);

    Ok((StatusCode::OK, headers, Bytes::from(buffer)).into_response())
}

pub async fn scheduler_asset_names() -> Response
{
    let asset_names = Asset::convert_to_asset_names();

    Json(asset_names).into_response()
}

// This should I think that the best thing to do here is to make the
// system work with the correct api. The Handlers should make the
// messages for the Actors based on the Request. That means that it
// becomes natural for the types to work with DTO objects on the
// frontend. Where should these reside? I think that the contracts
// crate is the best approach. So you make the api crate rely on
// the orchestrator, and the `constracts` crates.
// WARN [ ]
// This function might make sense
// pub async fn handle_orchestrator_request<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     orchestrator_request: OrchestratorRequest,
// ) -> Result<HttpResponse, ErrorRe>
// where
//     Ss: SystemSolutionTrait,
// {
//     event!(Level::INFO, orchestrator_request = ?orchestrator_request);
//     orchestrator
//         .orchestrator_requests(orchestrator_request)
//         .await
// }

// NOTE
// You have made this function, but you do not know where to put the
// trait bounds. This is a crucial lesson. You need to take good
// stock of this for the whole thing to work.
// So here you simply send a single message from the enum and you
// then get a return value. This handler is actually a helper
// function.
//
//
//
pub async fn orchestrator_status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Result<Response>
{
    Ok(Json(orchestrator.actor_registries.lock().unwrap().len()).into_response())
}

pub async fn get_days(
    orchestrator: State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Result<Response>
{
    let json = Json(OrchestratorRequest::GetPeriods);
    orchestrator_helper(orchestrator, json).await
}

/// This is a helper function for routing message for the orchestrator. Each
/// `handler` function should ideally use this to send a [`OrchestratorRequest`]
/// to the orchestrator.
#[debug_handler]
pub async fn orchestrator_helper(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Json(orchestrator_request): Json<OrchestratorRequest>,
) -> Result<Response>
{
    let response = orchestrator
        // So all the handling logic should reside in the Orchestrator itself.
        .handle(orchestrator_request)
        .await
        .unwrap();
    // This is horrible. I do not think that you should work on this
    //so late in the evening. Instead focus on doing
    //what actually Json(response)

    Ok(Json(response).into_response())
}
