// FIX
// This is a wrong way to import dependencies. It should be refactored.
// So now you have to decide what the best approach is to proceed here. I think
// that you should strive for making the sys
// QUESTION [ ]
// Should you make this work with the
// Where should the system messages be found?
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::web;
use ordinator_orchestrator::Orchestrator;
use tracing::Level;
use tracing::event;

// INFO
// So the idea is that all the functions should be separate. And the endpoints
// should simply call the different functions. What is the difference between
// the orchesatrator functions and handlers? The orchestrator simply has
// `Communication`s, `SchedulingEnvironment` `SystemSolutions` that you can use.
// This is what the orchestrator is. The remaining things should come from the
// handlers. They should provide the information that the orchestrator
// needs to do what it is supposed to do.
pub async fn handle_orchestrator_message(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    reg: HttpRequest,
) -> Result<HttpResponse, actix_web::Error>
{
    let mut orchestrator = orchestrator.lock().unwrap();

    Ok(orchestrator
        .handle_orchestrator_request(orchestrator_request)
        .await?)
}
#[allow(clippy::await_holding_lock)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    // Should all these System Messages be split into `async` functions?
    // FIX
    // This means that the SystemMessages should be broken down
    system_messages: web::Json<SystemMessages>,
) -> Result<HttpResponse, actix_web::Error>
{
    event!(Level::INFO, orchestrator_request = ?system_messages);
    match system_messages.into_inner() {
        SystemMessages::Orchestrator(orchestrator_request) => {}
        SystemMessages::Strategic(strategic_request) => {
            let asset = strategic_request.asset;
            let orchestrator_guard = orchestrator.lock().unwrap();

            let strategic = &orchestrator_guard
                .agent_registries
                .get(&asset)
                .unwrap()
                .strategic_agent_sender;

            strategic
                .sender
                .send(crate::agents::ActorMessage::Actor(
                    strategic_request.strategic_request_message,
                ))
                .map_err(actix_web::error::ErrorInternalServerError)?;

            let response = strategic
                .receiver
                .recv()
                .map_err(actix_web::error::ErrorInternalServerError)?;
            drop(orchestrator_guard);

            let strategic_response_message = match response {
                Ok(message) => message,
                Err(e) => {
                    let error = format!("{:?}", e.context("http request could not be completed"));
                    return Ok(HttpResponse::BadRequest().body(error));
                }
            };

            let strategic_response = StrategicResponse::new(asset, strategic_response_message);

            let system_message = SystemResponses::Strategic(strategic_response);

            Ok(HttpResponse::Ok().json(system_message))
        }
        SystemMessages::Tactical(tactical_request) => {
            let orchestrator = orchestrator.lock().unwrap();

            Ok(orchestrator
                .handle_tactical_request(tactical_request)
                .await?)
        }
        SystemMessages::Supervisor(supervisor_request) => {
            let orchestrator = orchestrator.lock().unwrap();

            Ok(orchestrator
                .handle_supervisor_request(supervisor_request)
                .await?)
        }
        SystemMessages::Operational(operational_request) => {
            let orchestrator = orchestrator.lock().unwrap();

            Ok(orchestrator
                .handle_operational_request(operational_request)
                .await?)
        }
        SystemMessages::Sap => Ok(HttpResponse::Ok().json(SystemResponses::Sap)),
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;

    use chrono::Utc;
    use shared_types::agents::tactical::Days;
    use shared_types::agents::tactical::TacticalResources;
    use shared_types::scheduling_environment::time_environment::day::Day;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    #[test]
    fn test_day_serialize()
    {
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
