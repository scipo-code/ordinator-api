mod orchestrator;
mod strategic;
mod supervisor;
mod tactical;
mod technician;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use orchestrator::export_xlsx;
use orchestrator::orchestrator_api_scope;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;
use strategic::scheduler_nest;
use supervisor::supervisor_routes;
use tactical::tactical_route;

use crate::handlers::orchestrator_handlers::scheduler_asset_names;

pub async fn api_scope(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> Router<Arc<Orchestrator<TotalSystemSolution>>>
{
    Router::new()
        .nest("/scheduler/", scheduler_nest(state.clone()).await)
        .nest("/export", export_xlsx(state.clone()).await)
        .nest("/orchestrator", orchestrator_api_scope(state.clone()).await)
        .nest("/tactical", tactical_route(state.clone()).await)
        .nest("/supervisor", supervisor_routes(state.clone()).await)
        .route("/assets", get(scheduler_asset_names))
    // .nest("/supervisor", router)
}
// pub fn api_scope() -> actix_web::Scope
// {
//     // Add routes like shown below
//     //
//     web::scope("/api/v1")
//     // .route(
//     //     "/scheduler/export/{asset}",
//     //     web::get().to(scheduler_excel_export),
//     // )
//     // .route("/scheduler/assets", web::get().to(scheduler_asset_names))
// }
//
// ISSUE #131 TODO [ ]
// Replace the `SystemMessages` structure with routers instead.
// pub enum SystemMessages {
//     Orchestrator(OrchestratorRequest),
//     Strategic(StrategicRequest),
//     Tactical(TacticalRequest),
//     Supervisor(SupervisorRequest),
//     Operational(OperationalRequest),
//     Sap,
// }

// #[derive(Serialize)]
// pub enum SystemResponses {
//     Orchestrator(OrchestratorResponse),
//     Strategic(StrategicResponse),
//     Tactical(TacticalResponse),
//     Supervisor(SupervisorResponse),
//     Operational(OperationalResponse),
//     Export,
//     Sap,
// }
