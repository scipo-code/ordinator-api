mod strategic;
mod supervisor;
mod tactical;
mod technician;

use std::sync::Arc;
use std::sync::Mutex;

use axum::Router;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;
use strategic::scheduler_nest;

pub async fn api_scope(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> Router<Arc<Orchestrator<TotalSystemSolution>>>
{
    Router::new().nest("scheduler/", scheduler_nest(state).await)
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
