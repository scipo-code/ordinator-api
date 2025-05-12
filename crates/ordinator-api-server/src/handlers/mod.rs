pub mod http_to_scheduling_system;
mod operational_handlers;
pub mod orchestrator_handlers;
mod supervisor_handlers;
mod tactical_handlers;
use anyhow::Result;
use axum::response::Response;

// use crate::orchestrator::Orchestrator;

pub async fn scheduler_excel_export(// WARN link to application data
    // orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    // WARN url query parameters
    // asset: web::Path<Asset>,
) -> Result<Response, axum::Error>
{
    Ok(Response::new("TODO".into()))
}
