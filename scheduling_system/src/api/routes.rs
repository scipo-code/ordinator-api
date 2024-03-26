use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_messages::LevelOfDetail;
use shared_messages::{orchestrator::OrchestratorRequest, SystemMessages};
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use tracing::instrument;
use tracing_subscriber::EnvFilter;

use crate::agents::orchestrator::Orchestrator;

#[instrument(level = "info", skip_all)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> Result<HttpResponse> {
    dbg!();
    match payload.0 {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let mut mutux_guard = orchestrator.lock().unwrap();
            let response = mutux_guard.handle(orchestrator_request).await;
            let http_response = HttpResponse::Ok()
                .insert_header(header::ContentType::plaintext())
                .body(response);
            Ok(http_response)
            // Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE"))
        }
        SystemMessages::Strategic(strategic_request) => {
            let strategic_agent_addr = orchestrator
                .lock()
                .unwrap()
                .agent_registry
                .strategic_agent_addr();

            let response = strategic_agent_addr.send(strategic_request).await;
            match response {
                Ok(response) => match response {
                    Ok(response) => {
                        let http_response = HttpResponse::Ok()
                            .insert_header(header::ContentType::plaintext())
                            .body(response);
                        Ok(http_response)
                    }
                    Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
                },
                Err(_) => Ok(HttpResponse::BadRequest().json("STRATEGIC: FAILURE")),
            }
        }
        SystemMessages::Tactical(tactical_request) => {
            let tactical_agent_addr = orchestrator
                .lock()
                .unwrap()
                .agent_registry
                .tactical_agent_addr();
            let response = tactical_agent_addr.send(tactical_request).await;

            match response {
                Ok(response) => match response {
                    Ok(response) => {
                        let http_response = HttpResponse::Ok()
                            .insert_header(header::ContentType::plaintext())
                            .body(response);
                        Ok(http_response)
                    }
                    Err(_) => Ok(HttpResponse::BadRequest().json("TACTICAL: FAILURE")),
                },
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

impl Orchestrator {
    #[instrument(level = "info", skip_all)]
    async fn handle(&mut self, msg: OrchestratorRequest) -> String {
        match msg {
            OrchestratorRequest::GetAgentStatus => {
                let strategic_agent_addr = self.agent_registry.strategic_agent_addr();
                let tactical_agent_addr = self.agent_registry.tactical_agent_addr();

                let mut buffer = String::new();

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
                for (_id, addr) in self.agent_registry.supervisor_agent_addrs.iter() {
                    let supervisor_agent_status =
                        addr.send(shared_messages::StatusMessage {}).await;
                    writeln!(buffer, "    {:?}", supervisor_agent_status).unwrap();
                }

                writeln!(buffer, "Operational agents:").unwrap();
                for (_id, addr) in self.agent_registry.operational_agent_addrs.iter() {
                    let operational_agent_status =
                        addr.send(shared_messages::StatusMessage {}).await;
                    writeln!(buffer, "    {:?}", operational_agent_status).unwrap();
                }

                buffer
            }
            OrchestratorRequest::GetWorkOrderStatus(work_order_number, level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                if let Some(work_order) = cloned_work_orders.inner.get(&work_order_number) {
                    match level_of_detail {
                        LevelOfDetail::Normal => work_order.to_string_normal(),
                        LevelOfDetail::Verbose => work_order.to_string_verbose(),
                    }
                } else {
                    "Work order not found".to_string()
                }
            }
            OrchestratorRequest::GetWorkOrdersState(level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                match level_of_detail {
                    LevelOfDetail::Normal => cloned_work_orders.to_string(),
                    LevelOfDetail::Verbose => "Not implemented".to_string(),
                }
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_strategic_periods();

                let periods_string: String = periods
                    .iter()
                    .map(|period| period.period_string())
                    .collect::<Vec<String>>()
                    .join(",");

                periods_string
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard.clone_tactical_days();

                let days_string: String = days
                    .iter()
                    .map(|day| day.date().to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                days_string
            }
            OrchestratorRequest::CreateSupervisorAgent(id) => {
                let tactical_agent_addr = self.agent_registry.tactical_agent_addr();
                let supervisor_agent_addr = self
                    .agent_factory
                    .build_supervisor_agent(id.clone(), tactical_agent_addr);

                self.agent_registry
                    .add_supervisor_agent(id.clone(), supervisor_agent_addr.clone());
                format!("Supervisor agent created with id {}", id)
            }
            OrchestratorRequest::DeleteSupervisorAgent(id_string) => {
                let id = self.agent_registry.supervisor_by_id_string(id_string);

                let supervisor_agent_addr = self.agent_registry.supervisor_agent_addr(id.clone());

                supervisor_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registry.supervisor_agent_addrs.remove(&id);

                format!("Supervisor agent deleted with id {}", id)
            }
            OrchestratorRequest::CreateOperationalAgent(id) => {
                let supervisor_agent_addr = self
                    .agent_registry
                    .supervisor_agent_addr_by_resource(&id.1[0].clone());

                let operational_agent_addr = self
                    .agent_factory
                    .build_operational_agent(id.clone(), supervisor_agent_addr);

                self.agent_registry
                    .add_operational_agent(id.clone(), operational_agent_addr.clone());

                format!("Operational agent created with id {}", id)
            }
            OrchestratorRequest::DeleteOperationalAgent(id_string) => {
                let id = self
                    .agent_registry
                    .supervisor_by_id_string(id_string.clone());

                let operational_agent_addr = self.agent_registry.operational_agent_addr(id.clone());

                operational_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registry.operational_agent_addrs.remove(&id);

                format!("Operational agent deleted  with id {}", id_string)
            }
            OrchestratorRequest::SetLogLevel(log_level) => {
                dbg!();
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                format!("Log level {}", log_level.to_level_string())
            }
            OrchestratorRequest::SetProfiling(log_level) => {
                dbg!();
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                format!("Profiling level {}", log_level.to_level_string())
            }
        }
    }
}
