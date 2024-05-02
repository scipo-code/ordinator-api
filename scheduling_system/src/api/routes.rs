use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_messages::strategic::strategic_response_status::{WorkOrderResponse, WorkOrdersStatus};
use shared_messages::LevelOfDetail;
use shared_messages::{orchestrator::OrchestratorRequest, SystemMessages};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use tracing::{instrument, warn};
use tracing_subscriber::EnvFilter;

use crate::agents::orchestrator::Orchestrator;
use shared_messages::models::WorkOrders;

#[allow(clippy::await_holding_lock)]
#[instrument(level = "info", skip_all)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> Result<HttpResponse> {
    match payload.0 {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let response = {
                orchestrator
                    .lock()
                    .unwrap()
                    .handle(orchestrator_request)
                    .await
            };

            match response {
                Ok(response) => {
                    let http_response = HttpResponse::Ok()
                        .insert_header(header::ContentType::json())
                        .body(response);
                    Ok(http_response)
                }
                Err(err) => Ok(HttpResponse::BadRequest().json(err)),
            }
        }
        SystemMessages::Strategic(strategic_request) => {
            let strategic_agent_addr = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(strategic_request.asset())
            {
                Some(agent_registry) => agent_registry.strategic_agent_addr(),
                None => {
                    warn!("Strategic agent not created for the asset");
                    return Ok(HttpResponse::BadRequest()
                        .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET"));
                }
            };

            let response = strategic_agent_addr
                .send(strategic_request.strategic_request_message)
                .await;
            match response {
                Ok(response) => match response {
                    Ok(response) => {
                        let http_response = HttpResponse::Ok()
                            .insert_header(header::ContentType::json())
                            .body(response);
                        Ok(http_response)
                    }
                    Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
                },
                Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
            }
        }
        SystemMessages::Tactical(tactical_request) => {
            let agent_registry_for_asset = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(&tactical_request.asset)
            {
                Some(asset) => asset.tactical_agent_addr(),
                None => {
                    warn!("Tactical agent not created for the asset");
                    return Ok(HttpResponse::BadRequest()
                        .json("TACTICAL: TACTICAL AGENT NOT INITIALIZED FOR THE ASSET"));
                }
            };

            let response = agent_registry_for_asset
                .send(tactical_request.tactical_request_message)
                .await;

            match response {
                Ok(response) => match response {
                    Ok(response) => {
                        let http_response = HttpResponse::Ok()
                            .insert_header(header::ContentType::json())
                            .body(response);
                        Ok(http_response)
                    }
                    Err(_) => Ok(HttpResponse::BadRequest().json("TACTICAL: FAILURE")),
                },
                Err(_) => Ok(HttpResponse::BadRequest().json("TACTICAL: FAILURE")),
            }
        }
        SystemMessages::Supervisor(supervisor_request) => {
            let supervisor_agent_addrs = match orchestrator
                .lock()
                .unwrap()
                .agent_registries
                .get(&supervisor_request.asset)
            {
                Some(agent_registry) => agent_registry.supervisor_agent_addrs.clone(),
                None => {
                    warn!("Supervisor agent not created for the asset");
                    return Ok(HttpResponse::BadRequest()
                        .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE ASSET"));
                }
            };

            let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.2.as_ref().unwrap() == &supervisor_request.main_work_center)
                .unwrap()
                .1;

            let response = supervisor_agent_addr
                .send(supervisor_request.supervisor_request_message)
                .await;

            match response {
                Ok(response) => match response {
                    Ok(response) => {
                        let http_response = HttpResponse::Ok()
                            .insert_header(header::ContentType::json())
                            .body(response);
                        Ok(http_response)
                    }
                    Err(_) => Ok(HttpResponse::BadRequest().json("SUPERVISOR: FAILURE")),
                },
                Err(_) => Ok(HttpResponse::BadRequest().json("SUPERVISOR: FAILURE")),
            }
        }
        SystemMessages::Operational => {
            Ok(HttpResponse::Ok().json("OPERATIONAL: IMPLEMENT SEND LOGIC"))
        }
        SystemMessages::Sap => Ok(HttpResponse::Ok().json("SAP: IMPLEMENT SEND LOGIC")),
    }
}

impl Orchestrator {
    #[instrument(level = "info", skip_all)]
    async fn handle(&mut self, msg: OrchestratorRequest) -> Result<String, String> {
        match msg {
            OrchestratorRequest::GetAgentStatus => {
                let mut buffer = String::new();
                for asset in self.agent_registries.keys() {
                    let strategic_agent_addr = self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .strategic_agent_addr();
                    let tactical_agent_addr = self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .tactical_agent_addr();

                    let strategic_agent_status = strategic_agent_addr
                        .send(shared_messages::StatusMessage {})
                        .await;
                    writeln!(buffer, "Strategic agents:").unwrap();
                    writeln!(buffer, "    {:?}", strategic_agent_status).unwrap();

                    let tactical_agent_status = tactical_agent_addr
                        .send(shared_messages::StatusMessage {})
                        .await;

                    writeln!(buffer, "Tactical agents:").unwrap();
                    writeln!(buffer, "    {:?}", tactical_agent_status).unwrap();

                    writeln!(buffer, "Supervisor agents:").unwrap();
                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .supervisor_agent_addrs
                        .iter()
                    {
                        let supervisor_agent_status =
                            addr.send(shared_messages::StatusMessage {}).await;
                        writeln!(buffer, "    {:?}", supervisor_agent_status).unwrap();
                    }

                    writeln!(buffer, "Operational agents:").unwrap();
                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .operational_agent_addrs
                        .iter()
                    {
                        let operational_agent_status =
                            addr.send(shared_messages::StatusMessage {}).await;
                        writeln!(buffer, "    {:?}", operational_agent_status).unwrap();
                    }
                }

                Ok(buffer)
            }
            OrchestratorRequest::GetWorkOrderStatus(work_order_number, level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                if let Some(work_order) = cloned_work_orders.inner.get(&work_order_number) {
                    match level_of_detail {
                        LevelOfDetail::Normal => Ok(work_order.to_string_normal()),
                        LevelOfDetail::Verbose => Ok(work_order.to_string_verbose()),
                    }
                } else {
                    Ok("Work order not found".to_string())
                }
            }
            OrchestratorRequest::GetWorkOrdersState(asset, level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders:WorkOrders = scheduling_environment_guard.clone_work_orders();
                let work_orders: WorkOrders = cloned_work_orders.inner.into_iter().filter(|wo| wo.1.work_order_info.functional_location.asset == asset).collect();

                let work_order_responses: HashMap<u32, WorkOrderResponse> = work_orders.inner.iter().map(|(work_order_number, work_order)| {
                    let work_order_response = WorkOrderResponse::new(
                        work_order.order_dates.earliest_allowed_start_period.clone(),
                        work_order.work_order_analytic.status_codes.awsc.clone(),
                        work_order.work_order_analytic.status_codes.sece.clone(),
                        work_order.work_order_info.revision.clone(),
                        work_order.work_order_info.work_order_type.clone(),
                        work_order.work_order_info.priority.clone(),
                        work_order.work_order_analytic.vendor.clone(),
                        work_order.work_order_analytic.status_codes.material_status.clone(),
                    );
                    (*work_order_number, work_order_response)
                
                }).collect();


                let work_orders_status = WorkOrdersStatus::new(work_order_responses);

                let message = serde_json::to_string(&work_orders_status).unwrap();
                Ok(message)
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_strategic_periods();

                let periods_string: String = periods
                    .iter()
                    .map(|period| period.period_string())
                    .collect::<Vec<String>>()
                    .join(",");

                Ok(periods_string)
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard.tactical_days();

                let days_string: String = days
                    .iter()
                    .map(|day| day.date().to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                Ok(days_string)
            }
            OrchestratorRequest::CreateSupervisorAgent(asset, id_string) => {
                let tactical_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .tactical_agent_addr();

                let supervisor_agent_addr = self.agent_factory.build_supervisor_agent(
                    asset.clone(),
                    id_string.clone(),
                    tactical_agent_addr,
                );

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_supervisor_agent(id_string.clone(), supervisor_agent_addr.clone());
                Ok(format!("Supervisor agent created with id {}", id_string))
            }
            OrchestratorRequest::DeleteSupervisorAgent(asset, id_string) => {
                let id = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string);

                let supervisor_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_agent_addr(id.clone());

                supervisor_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .supervisor_agent_addrs
                    .remove(&id);

                Ok(format!("Supervisor agent deleted with id {}", id))
            }
            OrchestratorRequest::CreateOperationalAgent(asset, id_string) => {
                let supervisor_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_agent_addr_by_resource(&id_string.1[0].clone());

                let operational_agent_addr = self
                    .agent_factory
                    .build_operational_agent(id_string.clone(), supervisor_agent_addr);

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_operational_agent(id_string.clone(), operational_agent_addr.clone());

                Ok(format!("Operational agent created with id {}", id_string))
            }
            OrchestratorRequest::DeleteOperationalAgent(asset, id_string) => {
                let id = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string.clone());

                let operational_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .operational_agent_addr(id.clone());

                operational_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .operational_agent_addrs
                    .remove(&id);

                Ok(format!("Operational agent deleted  with id {}", id_string))
            }
            OrchestratorRequest::SetLogLevel(log_level) => {
                dbg!();
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                Ok(format!("Log level {}", log_level.to_level_string()))
            }
            OrchestratorRequest::SetProfiling(log_level) => {
                dbg!();
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                Ok(format!("Profiling level {}", log_level.to_level_string()))
            }
            OrchestratorRequest::Export(asset) => {
                let agent_registry_for_asset = match self.agent_registries.get(&asset) {
                    Some(agent_registry) => agent_registry,
                    None => {
                        return Err(format!("Agent registry not found for the asset {}", asset));
                    }
                };

                let strategic_agent_solution = agent_registry_for_asset
                    .strategic_agent_addr
                    .send(shared_messages::SolutionExportMessage {})
                    .await;

                let tactical_agent_solution = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .tactical_agent_addr()
                    .send(shared_messages::SolutionExportMessage {})
                    .await;

                Ok(format!(
                    "{{\"strategic_agent_solution\": {}, \"tactical_agent_solution\": {}}}",
                    strategic_agent_solution.unwrap(),
                    tactical_agent_solution.unwrap()
                ))
            }
        }
    }
}
