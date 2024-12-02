use std::{collections::HashMap, fs::File, io::Read};

use actix_web::{http::StatusCode, HttpResponse};
use data_processing::excel_dumps::create_excel_dump;
use shared_types::operational::{
    operational_request_status::OperationalStatusRequest, OperationalRequest,
    OperationalRequestMessage, OperationalResponse, OperationalResponseMessage,
};
use shared_types::scheduling_environment::{
    time_environment::day::Day,
    work_order::{operation::ActivityNumber, WorkOrderNumber},
    worker_environment::resources::Id,
};
use shared_types::{
    orchestrator::OrchestratorRequest,
    strategic::{StrategicRequest, StrategicResponse},
    supervisor::{SupervisorRequest, SupervisorResponse},
    tactical::{TacticalRequest, TacticalResponse},
    SystemResponses,
};
use tracing::{event, Level};

use crate::agents::orchestrator::Orchestrator;

impl Orchestrator {
    pub async fn handle_orchestrator_request(
        &mut self,
        orchestrator_request: OrchestratorRequest,
    ) -> HttpResponse {
        event!(Level::INFO, orchestrator_request = ?orchestrator_request);
        let response = match orchestrator_request {
            OrchestratorRequest::Export(asset) => {
                let agent_registry_for_asset =
                        match self.agent_registries.get(&asset) {
                            Some(agent_registry) => agent_registry,
                            None => return HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE).body("The requested asset has not been initialized due to a lack of computing power. Please contact Kristoffer Madsen if you wish to have your Asset be part of the program"),
                        };

                let strategic_agent_solution = agent_registry_for_asset
                    .strategic_agent_addr
                    .send(shared_types::SolutionExportMessage {})
                    .await;

                let tactical_agent_solution = self
                    .arc_swap_shared_solutions
                    .get(&asset)
                    .unwrap()
                    .0
                    .load()
                    .tactical
                    .tactical_days
                    .iter()
                    .filter(|(_, d)| d.is_some())
                    .map(|(won, opt_acn_tac)| (won, opt_acn_tac.as_ref().unwrap()))
                    .map(|(won, acn_tac)| {
                        (
                            *won,
                            acn_tac
                                .iter()
                                .map(|(acn, tac)| {
                                    (*acn, tac.scheduled.first().as_ref().unwrap().0.clone())
                                })
                                .collect::<HashMap<ActivityNumber, Day>>(),
                        )
                    })
                    .collect::<HashMap<WorkOrderNumber, HashMap<ActivityNumber, Day>>>();

                let scheduling_environment_lock = self.scheduling_environment.lock().unwrap();

                let work_orders = scheduling_environment_lock.work_orders().clone();
                drop(scheduling_environment_lock);

                let xlsx_filename = create_excel_dump(
                    asset.clone(),
                    work_orders,
                    strategic_agent_solution.unwrap().unwrap(),
                    tactical_agent_solution,
                )
                .unwrap();

                let mut buffer = Vec::new();

                let mut file = File::open(&xlsx_filename).unwrap();

                file.read_to_end(&mut buffer).unwrap();

                std::fs::remove_file(xlsx_filename).expect("The XLSX file could not be deleted");

                let filename = format!("ordinator_xlsx_dump_for_{}", asset);
                let http_header = format!("attachment; filename={}", filename,);

                return HttpResponse::Ok()
                    .content_type(
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    )
                    .insert_header(("Content-Disposition", http_header))
                    .body(buffer);
            }
            _ => self.handle(orchestrator_request).await,
        };

        let system_responses = SystemResponses::Orchestrator(response.unwrap());
        HttpResponse::Ok().json(system_responses)
    }

    pub async fn handle_strategic_request(
        &self,
        strategic_request: StrategicRequest,
    ) -> HttpResponse {
        event!(Level::INFO, strategic_request = ?strategic_request);
        let strategic_agent_addr = match self.agent_registries.get(strategic_request.asset()) {
            Some(agent_registry) => agent_registry.strategic_agent_addr.clone(),
            None => {
                return HttpResponse::BadRequest()
                    .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET");
            }
        };

        let response = match strategic_agent_addr
            .send(strategic_request.strategic_request_message.clone())
            .await
            .unwrap()
        {
            Ok(response) => response,
            Err(e) => {
                return HttpResponse::InternalServerError().body(e.root_cause().to_string());
            }
        };

        let strategic_response =
            StrategicResponse::new(strategic_request.asset().clone(), response);
        let system_message = SystemResponses::Strategic(strategic_response);
        HttpResponse::Ok().json(system_message)
    }

    pub async fn handle_tactical_request(&self, tactical_request: TacticalRequest) -> HttpResponse {
        event!(Level::INFO, tactical_request = ?tactical_request);
        let agent_registry_for_asset = match self.agent_registries.get(&tactical_request.asset) {
            Some(asset) => asset.tactical_agent_addr.clone(),
            None => {
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
        let system_responses = SystemResponses::Tactical(tactical_response);
        HttpResponse::Ok().json(system_responses)
    }
    pub async fn handle_supervisor_request(
        &self,
        supervisor_request: SupervisorRequest,
    ) -> HttpResponse {
        event!(Level::INFO, supervisor_request = ?supervisor_request);
        let supervisor_agent_addrs = match self.agent_registries.get(&supervisor_request.asset) {
            Some(agent_registry) => agent_registry.supervisor_agent_addrs.clone(),
            None => {
                return HttpResponse::BadRequest()
                    .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE ASSET");
            }
        };
        let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.2.as_ref().unwrap().id == supervisor_request.supervisor.to_string())
                .expect("This will error at somepoint you will need to handle if you have added additional supervisors")
                .1;

        let response = supervisor_agent_addr
            .send(supervisor_request.supervisor_request_message)
            .await
            .unwrap()
            .unwrap();

        let supervisor_response = SupervisorResponse::new(supervisor_request.asset, response);

        let system_responses = SystemResponses::Supervisor(supervisor_response);
        HttpResponse::Ok().json(system_responses)
    }
    pub async fn handle_operational_request(
        &self,
        operational_request: OperationalRequest,
    ) -> HttpResponse {
        event!(Level::INFO, operational_request = ?operational_request);
        let operational_response = match operational_request {
            OperationalRequest::GetIds(asset) => {
                let mut operational_ids_by_asset: Vec<Id> = Vec::new();
                self.agent_registries
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
                match self
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
                                    return HttpResponse::InternalServerError().body(e.to_string())
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

                let agent_registry_option = self.agent_registries.get(&asset);

                let agent_registry = match agent_registry_option {
                    Some(agent_registry) => agent_registry,
                    None => {
                        return HttpResponse::BadRequest()
                            .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET");
                    }
                };

                for operational_addr in agent_registry.operational_agent_addrs.values() {
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
        let system_responses = SystemResponses::Operational(operational_response);
        HttpResponse::Ok().json(system_responses)
    }
}
