use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Id;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::SupervisorRequestMessage;
use ordinator_orchestrator::SupervisorResponseMessage;
use ordinator_orchestrator::SupervisorStatusMessage::General;
use ordinator_orchestrator::TotalSystemSolution;

#[debug_handler]
pub async fn status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, supervisor_id)): Path<(Asset, Id)>,
) -> Result<Json<SupervisorResponseMessage>>
{
    let lock = orchestrator.actor_registries.lock().unwrap();
    let communication = lock
        .get(&asset)
        .with_context(|| format!("Asset {asset} is not present in the ActorRegistry"))
        .unwrap()
        .supervisor_agent_senders
        .get(&supervisor_id)
        .with_context(|| {
            format!(
                "Supervisor {supervisor_id} on Asset {asset} is not present in the ActorRegistry"
            )
        })
        .unwrap();
    communication
        .from_agent(SupervisorRequestMessage::Status(General))
        .unwrap();

    Ok(Json(communication.from_actor()))
}

// _ISSUE_ #000 means unassigned
// TODO [ ] ISSUE #000
// You should craft the needed requests here. You should not be working on the
// Making a general function to handle every type of request to each actor, is
// a good idea. You should make this after the system is up and running.
// pub async fn handle_supervisor_request<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     supervisor_request: SupervisorRequest,
// ) -> Result<HttpResponse, actix_web::Error>
// where
//     Ss: SystemSolutionTrait,
// {
//     event!(Level::INFO, supervisor_request = ?supervisor_request);
//     let supervisor_agent_addrs = match
// self.agent_registries.get(&supervisor_request.asset) {
//         Some(agent_registry) => &agent_registry.supervisor_agent_senders,
//         None => {
//             return Ok(HttpResponse::BadRequest()
//                 .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE
// ASSET"));         }
//     };
//     let supervisor_agent_addr = supervisor_agent_addrs
//                 .iter()
//                 .find(|(id, _)| id.0 ==
// supervisor_request.supervisor.to_string())                 .expect("This will
// error at somepoint you will need to handle if you have added additional
// supervisors")                 .1;

//     // This was the reason that we wanted the tokio runtime.
//     supervisor_agent_addr
//         .sender
//         .send(crate::agents::ActorMessage::Actor(
//             supervisor_request.supervisor_request_message,
//         ))
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let response = supervisor_agent_addr
//         .receiver
//         .recv()
//         .map_err(actix_web::error::ErrorInternalServerError)?
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let supervisor_response =
// SupervisorResponse::new(supervisor_request.asset, response);

//     let system_responses = SystemResponses::Supervisor(supervisor_response);
//     Ok(HttpResponse::Ok().json(system_responses))
// }
