use actix_web::{web, HttpRequest, HttpResponse};
use shared_types::SystemMessages;
use shared_types::SystemResponses;

use std::sync::Arc;
use tracing::instrument;

use crate::agents::orchestrator::Orchestrator;

#[instrument(level = "info", skip_all)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<tokio::sync::Mutex<Orchestrator>>>,
    _req: HttpRequest,
    system_messages: web::Json<SystemMessages>,
) -> HttpResponse {
    match system_messages.into_inner() {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let mut orchestrator = orchestrator.lock().await;

            orchestrator
                .handle_orchestrator_request(orchestrator_request)
                .await
        }
        SystemMessages::Strategic(strategic_request) => {
            let orchestrator = orchestrator.lock().await;

            orchestrator
                .handle_strategic_request(strategic_request)
                .await
        }
        SystemMessages::Tactical(tactical_request) => {
            let orchestrator = orchestrator.lock().await;

            orchestrator.handle_tactical_request(tactical_request).await
        }
        SystemMessages::Supervisor(supervisor_request) => {
            let orchestrator = orchestrator.lock().await;

            orchestrator
                .handle_supervisor_request(supervisor_request)
                .await
        }
        SystemMessages::Operational(operational_request) => {
            let orchestrator = orchestrator.lock().await;

            orchestrator
                .handle_operational_request(operational_request)
                .await
        }
        SystemMessages::Sap => HttpResponse::Ok().json(SystemResponses::Sap),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use shared_types::{
        scheduling_environment::{
            time_environment::day::Day, work_order::operation::Work,
            worker_environment::resources::Resources,
        },
        tactical::{Days, TacticalResources},
    };

    #[test]
    fn test_day_serialize() {
        let mut hash_map_nested = HashMap::<Day, Work>::new();

        let mut hash_map = HashMap::<Resources, Days>::new();
        let day = Day::new(0, Utc::now());
        day.to_string();
        hash_map_nested.insert(day, Work::from(123.0));

        hash_map.insert(Resources::MtnMech, Days::new(hash_map_nested.clone()));
        let tactical_resources = TacticalResources::new(hash_map.clone());
        serde_json::to_string(&tactical_resources).unwrap();
    }
}
