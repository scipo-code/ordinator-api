use std::{collections::HashMap, fs::File, io::Read};

use actix_web::HttpResponse;
use anyhow::Context;
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
use shared_types::supervisor::{SupervisorRequest, SupervisorResponse};
use shared_types::{
    orchestrator::OrchestratorRequest,
    tactical::{TacticalRequest, TacticalResponse},
    SystemResponses,
};
use tracing::{event, Level};

use crate::agents::orchestrator::Orchestrator;

impl Orchestrator {
    pub async fn handle_orchestrator_request(
        &mut self,
        orchestrator_request: OrchestratorRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        event!(Level::INFO, orchestrator_request = ?orchestrator_request);
        let response = match orchestrator_request {
            OrchestratorRequest::Export(asset) => {
                let shared_solution = self
                    .arc_swap_shared_solutions
                    .get(&asset)
                    .with_context(|| {
                        format!(
                            "Could not retrieve the shared_solution for asset {:#?}",
                            asset
                        )
                    })
                    .map_err(|err| actix_web::error::ErrorInternalServerError(err))?
                    .0
                    .load();

                let strategic_agent_solution = shared_solution
                    .strategic
                    .strategic_scheduled_work_orders
                    .clone()
                    .into_iter()
                    .filter_map(|e| e.1.map(|t| (e.0, t)))
                    .collect::<HashMap<_, _>>();

                let tactical_agent_solution = self
                    .arc_swap_shared_solutions
                    .get(&asset)
                    .unwrap()
                    .0
                    .load()
                    .tactical
                    .tactical_scheduled_work_orders
                    .0
                    .iter()
                    .filter(|(_, tac_sch)| tac_sch.is_tactical())
                    .map(|(won, opt_acn_tac)| (won, opt_acn_tac.tactical_operations()))
                    .map(|(won, acn_tac)| {
                        (
                            *won,
                            acn_tac
                                .unwrap()
                                .0
                                .iter()
                                .map(|(acn, tac)| {
                                    (*acn, tac.scheduled.first().as_ref().unwrap().0.clone())
                                })
                                .collect::<HashMap<ActivityNumber, Day>>(),
                        )
                    })
                    .collect::<HashMap<WorkOrderNumber, HashMap<ActivityNumber, Day>>>();

                let scheduling_environment_lock = self.scheduling_environment.lock().unwrap();

                let work_orders = scheduling_environment_lock.work_orders.clone();
                drop(scheduling_environment_lock);

                let xlsx_filename = create_excel_dump(
                    asset.clone(),
                    work_orders,
                    shared_types::AgentExports::Strategic(strategic_agent_solution),
                    tactical_agent_solution,
                )
                .unwrap();

                let mut buffer = Vec::new();

                let mut file = File::open(&xlsx_filename).unwrap();

                file.read_to_end(&mut buffer).unwrap();

                std::fs::remove_file(xlsx_filename).expect("The XLSX file could not be deleted");

                let filename = format!("ordinator_xlsx_dump_for_{}", asset);
                let http_header = format!("attachment; filename={}", filename,);

                return Ok(HttpResponse::Ok()
                    .content_type(
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    )
                    .insert_header(("Content-Disposition", http_header))
                    .body(buffer));
            }
            _ => self.handle(orchestrator_request).await,
        };

        let system_responses = SystemResponses::Orchestrator(response.unwrap());
        Ok(HttpResponse::Ok().json(system_responses))
    }

    // TODO: Move this out
    pub async fn handle_tactical_request(
        &self,
        tactical_request: TacticalRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        let agent_registry_for_asset = match self.agent_registries.get(&tactical_request.asset) {
            Some(agent_registry) => &agent_registry.tactical_agent_sender,
            None => {
                return Ok(HttpResponse::BadRequest()
                    .json("TACTICAL: TACTICAL AGENT NOT INITIALIZED FOR THE ASSET"));
            }
        };

        agent_registry_for_asset
            .sender
            .send(crate::agents::AgentMessage::Actor(
                tactical_request.tactical_request_message,
            ))
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        let response = agent_registry_for_asset
            .receiver
            .recv()
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        let tactical_response = TacticalResponse::new(tactical_request.asset, response);
        let system_responses = SystemResponses::Tactical(tactical_response);
        Ok(HttpResponse::Ok().json(system_responses))
    }

    // TODO: Move this out
    pub async fn handle_supervisor_request(
        &self,
        supervisor_request: SupervisorRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        event!(Level::INFO, supervisor_request = ?supervisor_request);
        let supervisor_agent_addrs = match self.agent_registries.get(&supervisor_request.asset) {
            Some(agent_registry) => &agent_registry.supervisor_agent_senders,
            None => {
                return Ok(HttpResponse::BadRequest()
                    .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE ASSET"));
            }
        };
        let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.2.as_ref().unwrap().id == supervisor_request.supervisor.to_string())
                .expect("This will error at somepoint you will need to handle if you have added additional supervisors")
                .1;

        supervisor_agent_addr
            .sender
            .send(crate::agents::AgentMessage::Actor(
                supervisor_request.supervisor_request_message,
            ))
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        let response = supervisor_agent_addr
            .receiver
            .recv()
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        let supervisor_response = SupervisorResponse::new(supervisor_request.asset, response);

        let system_responses = SystemResponses::Supervisor(supervisor_response);
        Ok(HttpResponse::Ok().json(system_responses))
    }

    // TODO: Move this out
    pub async fn handle_operational_request(
        &self,
        operational_request: OperationalRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        let operational_response = match operational_request {
            OperationalRequest::GetIds(asset) => {
                let mut operational_ids_by_asset: Vec<Id> = Vec::new();
                self.agent_registries
                    .get(&asset)
                    .expect("This error should be handled higher up")
                    .operational_agent_senders
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
                    Some(agent_communication) => {
                        agent_communication
                            .sender
                            .send(crate::agents::AgentMessage::Actor(operational_request_message))
                            .context("Could not await the message sending, theard problems are the most likely")
                            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

                        let response = agent_communication
                            .receiver
                            .recv()
                            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?
                            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

                        OperationalResponse::OperationalState(response)
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
                        return Ok(HttpResponse::BadRequest()
                            .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET"));
                    }
                };

                for operational_addr in agent_registry.operational_agent_senders.values() {
                    operational_addr
                        .sender
                        .send(crate::agents::AgentMessage::Actor(
                            operational_request_message.clone(),
                        ))
                        .unwrap();
                }

                for operational_addr in agent_registry.operational_agent_senders.values() {
                    let response = operational_addr.receiver.recv().unwrap().unwrap();

                    operational_responses.push(response);
                }
                OperationalResponse::AllOperationalStatus(operational_responses)
            }
        };
        let system_responses = SystemResponses::Operational(operational_response);
        Ok(HttpResponse::Ok().json(system_responses))
    }
}
