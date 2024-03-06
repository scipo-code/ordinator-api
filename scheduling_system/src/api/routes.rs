use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_messages::{orchestrator::OrchestratorRequest, SystemMessages};
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use crate::agents::orchestrator::Orchestrator;

pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> Result<HttpResponse> {
    match payload.0 {
        SystemMessages::Orchestrator(orchestrator_request) => {
            let mut mutux_guard = orchestrator.lock().unwrap();
            let response = mutux_guard.handle(orchestrator_request).await;
            Ok(HttpResponse::Ok().json(response))
        }
        SystemMessages::Strategic(strategic_request) => {
            let strategic_agent_addr = orchestrator
                .lock()
                .unwrap()
                .agent_registry
                .get_strategic_agent_addr();

            let response = strategic_agent_addr.send(strategic_request).await;
            match response {
                Ok(response) => match response {
                    Ok(response) => Ok(HttpResponse::Ok().json(response)),
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
                .get_tactical_agent_addr();

            let response = tactical_agent_addr.send(tactical_request).await;
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

impl Orchestrator {
    async fn handle(&mut self, msg: OrchestratorRequest) -> String {
        match msg {
            OrchestratorRequest::GetAgentStatus => {
                let strategic_agent_addr = self.agent_registry.get_strategic_agent_addr();
                let tactical_agent_addr = self.agent_registry.get_tactical_agent_addr();

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
            OrchestratorRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                if let Some(work_order_status) = cloned_work_orders.inner.get(&work_order_number) {
                    work_order_status.to_string()
                } else {
                    "Work order not found".to_string()
                }
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_periods();

                let periods_string: String = periods
                    .iter()
                    .map(|period| period.get_period_string())
                    .collect::<Vec<String>>()
                    .join(",");

                periods_string
            }
            OrchestratorRequest::CreateSupervisorAgent(id, resource) => {
                let supervisor_agent_addr = self
                    .agent_factory
                    .build_supervisor_agent(id.clone(), resource.clone());

                self.agent_registry
                    .add_supervisor_agent(id.clone(), supervisor_agent_addr.clone());
                format!("Supervisor agent created with id {}", id)
            }
            OrchestratorRequest::DeleteSupervisorAgent(id) => {
                let supervisor_agent_addr =
                    self.agent_registry.get_supervisor_agent_addr(id.clone());

                supervisor_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registry.supervisor_agent_addrs.remove(&id);

                format!("Supervisor agent deleted with id {}", id)
            }
            OrchestratorRequest::CreateOperationalAgent(id, resources) => {
                let operational_agent_addr = self
                    .agent_factory
                    .build_operational_agent(id.clone(), resources.clone());

                self.agent_registry
                    .add_operational_agent(id.clone(), operational_agent_addr.clone());

                format!("Operational agent created with id {}", id)
            }
            OrchestratorRequest::DeleteOperationalAgent(name) => {
                let operational_agent_addr =
                    self.agent_registry.get_operational_agent_addr(name.clone());

                operational_agent_addr.do_send(shared_messages::StopMessage {});

                self.agent_registry.operational_agent_addrs.remove(&name);

                format!("Operational agent deleted with id {}", name)
            }
        }
    }
}
