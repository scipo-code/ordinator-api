use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::HttpResponse;
use actix_web::web;
use anyhow::Context;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use ordinator_orchestrator::ActivityNumber;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Day;
use ordinator_orchestrator::OperationalRequestMessage;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::OrchestratorRequest;
use ordinator_orchestrator::SystemSolutionTrait;
use ordinator_orchestrator::TotalSystemSolution;
use ordinator_orchestrator::WorkOrderNumber;
use tracing::Level;
use tracing::event;

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

pub async fn scheduler_excel_export<Ss>(
    State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
    asset: web::Path<Asset>,
) -> Result<Response, axum::Error>
where
    Ss: SystemSolutionTrait,
{
    let (buffer, http_header) = orchestrator
        .lock()
        .unwrap()
        .export_xlsx_solution(asset.into_inner())?;

    let http_response = HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .insert_header(("Content-Disposition", http_header))
        .body(buffer);

    Ok(http_response)
}

pub async fn scheduler_asset_names<'a, Ss>(
    orchestrator: State<Arc<Mutex<Orchestrator<Ss>>>>,
) -> Result<HttpResponse, axum::Error>
where
    Ss: SystemSolutionTrait,
{
    let asset_names = Asset::convert_to_asset_names();

    let http_response = HttpResponse::Ok().json(asset_names);

    Ok(http_response)
}

// This should I think that the best thing to do here is to make the
// system work with the correct api. The Handlers should make the
// messages for the Actors based on the Request. That means that it
// becomes natural for the types to work with DTO objects on the
// frontend. Where should these reside? I think that the contracts
// crate is the best approach. So you make the api crate rely on
// the orchestrator, and the `constracts` crates.
pub async fn handle_orchestrator_request<Ss>(
    orchestrator: web::Data<Arc<Mutex<Orchestrator<Ss>>>>,
    orchestrator_request: OrchestratorRequest,
) -> Result<HttpResponse, axum::Error>
where
    Ss: SystemSolutionTrait,
{
    event!(Level::INFO, orchestrator_request = ?orchestrator_request);
    orchestrator
        .orchestrator_requests(orchestrator_request)
        .await
}

pub async fn orchestrator_requests(
    State(orchestrator): State<Arc<Mutex<Orchestrator<TotalSystemSolution>>>>,
    Json(orchestrator_request): Json<OrchestratorRequest>,
) -> Result<Response, axum::Error>
{
    let response = match orchestrator_request {
        OrchestratorRequest::Export(asset) => {
            let (buffer, http_header) = export_xlsx_solution(orchestrator, asset)?;

            HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .insert_header(("Content-Disposition", http_header))
                .body(buffer)
        }
        _ => {
            // let response = orchestrator
            //     .lock()
            //     .unwrap()
            //     .handle(orchestrator_request)
            //     .await;
            // // This is horrible. I do not think that you should work on this
            // so late // in the evening. Instead focus on doing
            // what actually Json(response)
        }
    };

    Ok(response)
}

// You need to understand how to deal with Generics in a good way in the API of
// the function.
pub fn export_xlsx_solution<Ss>(
    orchestrator: Arc<Mutex<Orchestrator<Ss>>>,
    asset: Asset,
) -> Result<(Vec<u8>, String), axum::Error>
where
    Ss: SystemSolutionTrait,
{
    let shared_solution = orchestrator
        .arc_swap_shared_solutions
        .get(&asset)
        .with_context(|| {
            format!(
                "Could not retrieve the shared_solution for asset {:#?}",
                asset
            )
        })
        .map_err(axum::Error::new(StatusCode::NOT_FOUND))?
        .0
        .load();

    let strategic_agent_solution = shared_solution
        .strategic
        .strategic_scheduled_work_orders
        .clone()
        .into_iter()
        // .filter_map(|(won, opt_per)| opt_per.map(|per| (won, per)))
        .collect::<HashMap<_, _>>();
    let tactical_agent_solution = orchestrator
        .arc_swap_shared_solutions
        .get(&asset)
        .unwrap()
        .0
        .load()
        .tactical
        .tactical_work_orders
        .0
        .iter()
        .filter(|(_, tac_sch)| tac_sch.is_tactical())
        .map(|(won, opt_acn_tac)| (won, opt_acn_tac.tactical_operations()))
        .map(|(won, acn_tac)| {
            (
                *won,
                acn_tac
                    .unwrap()
                    .0
                    .iter()
                    .map(|(acn, tac)| (*acn, tac.scheduled.first().as_ref().unwrap().0.clone()))
                    .collect::<HashMap<ActivityNumber, Day>>(),
            )
        })
        .collect::<HashMap<WorkOrderNumber, HashMap<ActivityNumber, Day>>>();

    let scheduling_environment_lock = orchestrator
        .lock()
        .unwrap()
        .scheduling_environment
        .lock()
        .unwrap();
    let work_orders = scheduling_environment_lock.work_orders.clone();

    drop(scheduling_environment_lock);
    let xlsx_filename = create_excel_dump(
        asset.clone(),
        work_orders,
        strategic_agent_solution,
        tactical_agent_solution,
    )
    .unwrap();
    let mut buffer = Vec::new();
    let mut file = File::open(&xlsx_filename).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    std::fs::remove_file(xlsx_filename).expect("The XLSX file could not be deleted");
    let filename = format!("ordinator_xlsx_dump_for_{}", asset);
    let http_header = format!("attachment; filename={}", filename,);

    Ok((buffer, http_header))
}

// TODO: Move this out
// How should this look like?
//
