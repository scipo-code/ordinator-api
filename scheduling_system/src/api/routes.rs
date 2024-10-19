use actix_web::{web, HttpRequest, HttpResponse};
use data_processing::excel_dumps::create_excel_dump;
use shared_types::operational::operational_request_resource::OperationalResourceRequest;
use shared_types::operational::operational_request_status::OperationalStatusRequest;
use shared_types::operational::operational_response_scheduling::OperationalSchedulingResponse;
use shared_types::operational::{
    OperationalRequest, OperationalRequestMessage, OperationalResponse, OperationalResponseMessage,
    OperationalTarget,
};
use shared_types::orchestrator::OrchestratorRequest;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::strategic::StrategicResponse;
use shared_types::supervisor::SupervisorResponse;

use shared_types::tactical::TacticalResponse;
use shared_types::SystemMessages;
use shared_types::SystemResponses;

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tracing::{event, instrument, warn, Level};

use crate::agents::orchestrator::Orchestrator;

#[instrument(level = "info", skip_all)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> HttpResponse {
    let system_responses: SystemResponses = match payload.0 {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let response = match orchestrator_request {
                OrchestratorRequest::Export(asset) => {
                    let orchestrator_lock = orchestrator.lock().unwrap();
                    let agent_registry_for_asset =
                        orchestrator_lock.agent_registries.get(&asset).unwrap();

                    let strategic_agent_solution = agent_registry_for_asset
                        .strategic_agent_addr
                        .send(shared_types::SolutionExportMessage {})
                        .await;

                    let tactical_agent_solution = orchestrator_lock
                        .agent_registries
                        .get(&asset)
                        .unwrap()
                        .tactical_agent_addr
                        .send(shared_types::SolutionExportMessage {})
                        .await;

                    let scheduling_environment_lock =
                        orchestrator_lock.scheduling_environment.lock().unwrap();

                    let work_orders = scheduling_environment_lock.work_orders().clone();
                    drop(scheduling_environment_lock);

                    let xlsx_filename = create_excel_dump(
                        asset.clone(),
                        work_orders,
                        strategic_agent_solution.unwrap().unwrap(),
                        tactical_agent_solution.unwrap().unwrap(),
                    )
                    .unwrap();

                    let mut buffer = Vec::new();

                    let mut file = File::open(&xlsx_filename).unwrap();

                    file.read_to_end(&mut buffer).unwrap();

                    std::fs::remove_file(xlsx_filename)
                        .expect("The XLSX file could not be deleted");

                    let filename = format!("ordinator_xlsx_dump_for_{}", asset);
                    let http_header = format!("attachment; filename={}", filename,);

                    return HttpResponse::Ok()
                        .content_type(
                            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        )
                        .insert_header(("Content-Disposition", http_header))
                        .body(buffer);
                }
                _ => {
                    orchestrator
                        .lock()
                        .unwrap()
                        .handle(orchestrator_request)
                        .await
                }
            };

            SystemResponses::Orchestrator(response.unwrap())
        }
        SystemMessages::Strategic(strategic_request) => {
            let strategic_agent_addr = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(strategic_request.asset())
            {
                Some(agent_registry) => agent_registry.strategic_agent_addr.clone(),
                None => {
                    warn!("Strategic agent not created for the asset");
                    return HttpResponse::BadRequest()
                        .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET");
                }
            };

            let response = strategic_agent_addr
                .send(strategic_request.strategic_request_message.clone())
                .await
                .unwrap()
                .unwrap();

            let strategic_response =
                StrategicResponse::new(strategic_request.asset().clone(), response);
            SystemResponses::Strategic(strategic_response)
        }
        SystemMessages::Tactical(tactical_request) => {
            let agent_registry_for_asset = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(&tactical_request.asset)
            {
                Some(asset) => asset.tactical_agent_addr.clone(),
                None => {
                    warn!("Tactical agent not created for the asset");
                    return HttpResponse::BadRequest()
                        .json("TACTICAL: TACTICAL AGENT NOT INITIALIZED FOR THE ASSET");
                }
            };

            let response = agent_registry_for_asset
                .send(tactical_request.tactical_request_message)
                .await
                .unwrap()
                .unwrap();

            let tactical_response = TacticalResponse::new(tactical_request.asset, response);
            SystemResponses::Tactical(tactical_response)
        }
        SystemMessages::Supervisor(supervisor_request) => {
            event!(Level::WARN, "before the locking of the actor registry");
            let supervisor_agent_addrs = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(&supervisor_request.asset)
            {
                Some(agent_registry) => agent_registry.supervisor_agent_addrs.clone(),
                None => {
                    warn!("Supervisor agent not created for the asset");
                    return HttpResponse::BadRequest()
                        .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE ASSET");
                }
            };
            event!(Level::WARN, "agent registry found the correct supervisor");
            let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.2.as_ref().unwrap() == &supervisor_request.main_work_center)
                .unwrap()
                .1;

            event!(Level::WARN, "supervisor addr extracted");
            let response = supervisor_agent_addr
                .send(supervisor_request.supervisor_request_message)
                .await
                .unwrap()
                .unwrap();

            event!(Level::WARN, "response generated");
            let supervisor_response = SupervisorResponse::new(supervisor_request.asset, response);

            SystemResponses::Supervisor(supervisor_response)
        }
        SystemMessages::Operational(operational_request) => {
            let operational_response = match operational_request {
                OperationalRequest::GetIds(asset) => {
                    let mut operational_ids_by_asset: Vec<Id> = Vec::new();
                    orchestrator
                        .lock()
                        .unwrap()
                        .agent_registries
                        .get(&asset)
                        .expect("This error should be handled higher up")
                        .operational_agent_addrs
                        .keys()
                        .for_each(|ele| {
                            operational_ids_by_asset.push(ele.clone());
                        });

                    OperationalResponse::OperationalIds(operational_ids_by_asset)
                }

                OperationalRequest::ForOperationalAgent((
                    asset,
                    operational_id,
                    operational_request_message,
                )) => {
                    match orchestrator
                        .lock()
                        .unwrap()
                        .agent_registries
                        .get(&asset)
                        .expect("This error should be handled higher up")
                        .get_operational_addr(&operational_id)
                    {
                        Some(addr) => {
                            let operational_response_message = match addr.send(operational_request_message).await.expect("Could not await the message sending, theard problems are the most likely") {
                                Ok(operational_response_message) => {
                                    operational_response_message
                                },
                                Err(e) => {
                                    let operational_scheduling_message = OperationalSchedulingResponse::Error(e);
                                    OperationalResponseMessage::Scheduling(operational_scheduling_message)
                                },
                            };
                            OperationalResponse::OperationalState(operational_response_message)
                        }
                        None => OperationalResponse::NoOperationalAgentFound(operational_id),
                    }
                }
                OperationalRequest::AllOperationalStatus(asset) => {
                    let operational_request_status = OperationalStatusRequest::General;
                    let operational_request_message =
                        OperationalRequestMessage::Status(operational_request_status);
                    let mut operational_responses: Vec<OperationalResponseMessage> = vec![];

                    for operational_addr in orchestrator
                        .lock()
                        .unwrap()
                        .agent_registries
                        .get(&asset)
                        .expect("If this fails it means that the error handling should have been done further up")
                        .operational_agent_addrs
                        .values()
                    {
                        operational_responses.push(
                            operational_addr
                                .send(operational_request_message.clone())
                                .await
                                .unwrap()
                                .unwrap(),
                        )
                    }
                    OperationalResponse::AllOperationalStatus(operational_responses)
                }
            };
            SystemResponses::Operational(operational_response)
        }
        SystemMessages::Sap => SystemResponses::Sap,
    };

    HttpResponse::Ok().json(system_responses)
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
