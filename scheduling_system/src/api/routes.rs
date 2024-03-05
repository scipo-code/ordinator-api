use std::sync::{Arc, RwLock};

use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_messages::SystemMessages;

use crate::agents::orchestrator_agent::ActorRegistry;

pub async fn http_to_scheduling_system(
    rw_actor_registry: web::Data<Arc<RwLock<ActorRegistry>>>,
    req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> Result<HttpResponse> {
    let actor_registry = rw_actor_registry.read().unwrap();
    match payload.0 {
        SystemMessages::Orchestrator(status_input) => {
            let response = actor_registry
                .get_orchestrator_agent_addr()
                .send(status_input)
                .await;
            match response {
                Ok(response) => Ok(HttpResponse::Ok().json(response)),
                Err(_) => Ok(HttpResponse::BadRequest().json("ORCHESTRATOR: FAILURE")),
            }
        }
        SystemMessages::Strategic(strategic_request) => {
            let response = actor_registry
                .get_strategic_agent_addr()
                .send(strategic_request)
                .await;
            match response {
                Ok(response) => match response {
                    Ok(response) => Ok(HttpResponse::Ok().json(response)),
                    Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
                },
                Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
            }
        }
        SystemMessages::Tactical(tactical_request) => {
            let response = actor_registry
                .get_tactical_agent_addr()
                .send(tactical_request)
                .await;
            match response {
                Ok(response) => {
                    dbg!(response);
                    Ok(HttpResponse::Ok().json("TACTICAL: SUCCESS"))
                }
                Err(_) => Ok(HttpResponse::BadRequest().json("TACTICAL: FAILURE")),
            }
        }
        SystemMessages::Supervisor => {
            Ok(HttpResponse::Ok().json("OPERATIONAL: IMPLEMENT SEND LOGIC"))
        }
        SystemMessages::Operational => {
            Ok(HttpResponse::Ok().json("OPERATIONAL: IMPLEMENT SEND LOGIC"))
        }
        SystemMessages::Sap => Ok(HttpResponse::Ok().json("SAP: IMPLEMENT SEND LOGIC")),
    }
}
