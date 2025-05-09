pub mod http_to_scheduling_system;
pub mod orchestrator_handlers;
use actix_web::HttpResponse;
use anyhow::Result;

// use crate::orchestrator::Orchestrator;

pub async fn scheduler_excel_export(// WARN link to application data
    // orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    // WARN url query parameters
    // asset: web::Path<Asset>,
) -> Result<HttpResponse, actix_web::Error>
{
    Ok(HttpResponse::Ok().into())
}
