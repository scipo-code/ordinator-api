use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Context;
use shared_types::strategic::StrategicResponse;
use shared_types::SystemMessages;
use shared_types::SystemResponses;
use tracing::event;
use tracing::instrument::WithSubscriber;
use tracing::Level;

use std::sync::Arc;
use std::sync::Mutex;
use tracing::instrument;

use crate::agents::orchestrator::Orchestrator;

#[instrument(level = "info", skip_all)]
#[allow(clippy::await_holding_lock)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    system_messages: web::Json<SystemMessages>,
) -> HttpResponse {
    event!(Level::INFO, orchestrator_request = ?system_messages);
    match system_messages.into_inner() {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let mut orchestrator = orchestrator.lock().unwrap();

            orchestrator
                .handle_orchestrator_request(orchestrator_request)
                .await
        }
        SystemMessages::Strategic(strategic_request) => {
            let asset = strategic_request.asset;
            let orchestrator_guard = orchestrator.lock().unwrap();
            let strategic = orchestrator_guard
                .agent_registries
                .get(&asset)
                .unwrap()
                .strategic_agent_addr
                .clone();

            drop(orchestrator_guard);
            let response = strategic
                .send(strategic_request.strategic_request_message)
                .await;

            let strategic_response_message = match response
                .expect("Failed to send StrategicRequestMessage")
            {
                Ok(message) => message,
                Err(e) => {
                    let error = format!("{:?}", e.context("http request could not be completed"));
                    return HttpResponse::BadRequest().body(error);
                }
            };

            let strategic_response = StrategicResponse::new(asset, strategic_response_message);

            let system_message = SystemResponses::Strategic(strategic_response);

            HttpResponse::Ok().json(system_message)
        }
        SystemMessages::Tactical(tactical_request) => {
            let orchestrator = orchestrator.lock().unwrap();

            orchestrator.handle_tactical_request(tactical_request).await
        }
        SystemMessages::Supervisor(supervisor_request) => {
            let orchestrator = orchestrator.lock().unwrap();

            orchestrator
                .handle_supervisor_request(supervisor_request)
                .await
        }
        SystemMessages::Operational(operational_request) => {
            let orchestrator = orchestrator.lock().unwrap();

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
