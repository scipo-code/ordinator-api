use actix::dev::channel::AddressSender;
use actix::dev::Request;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use futures::future::join_all;
use shared_messages::models::work_order::WorkOrderNumber;
use shared_messages::models::worker_environment::resources::Id;
use shared_messages::operational::operational_request_status::OperationalStatusRequest;
use shared_messages::operational::operational_response_status::OperationalStatusResponse;
use shared_messages::operational::{OperationalInfeasibleCases, OperationalRequestMessage, OperationalResponse, OperationalResponseMessage, OperationalTarget};
use shared_messages::orchestrator::{AgentStatus, AgentStatusResponse, OrchestratorResponse};
use shared_messages::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_messages::strategic::strategic_response_status::{WorkOrderResponse, WorkOrdersStatus};
use shared_messages::strategic::{StrategicResponse, StrategicResponseMessage};
use shared_messages::supervisor::supervisor_response_status::SupervisorResponseStatus;
use shared_messages::supervisor::supervisor_status_message::SupervisorStatusMessage;
use shared_messages::supervisor::{SupervisorRequestMessage, SupervisorResponse};

use shared_messages::tactical::tactical_status_message::TacticalStatusMessage;
use shared_messages::tactical::{TacticalRequestMessage, TacticalResponse, TacticalResponseMessage};
use shared_messages::{Asset, SystemResponses};
use shared_messages::{orchestrator::OrchestratorRequest, SystemMessages};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{instrument, warn};
use tracing_subscriber::EnvFilter;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::orchestrator::Orchestrator;
use crate::agents::UpdateWorkOrderMessage;
use shared_messages::models::WorkOrders;

#[instrument(level = "info", skip_all)]
pub async fn http_to_scheduling_system(
    orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    _req: HttpRequest,
    payload: web::Json<SystemMessages>,
) -> HttpResponse {
    let system_responses: SystemResponses = match payload.0 {
         SystemMessages::Orchestrator(orchestrator_request) => {
            let response = {
                orchestrator
                    .lock()
                    .unwrap()
                    .handle(orchestrator_request)
                    .await
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

            let strategic_response = StrategicResponse::new(strategic_request.asset().clone(), response);
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

            let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.2.as_ref().unwrap() == &supervisor_request.main_work_center)
                .unwrap()
                .1;

            let response = supervisor_agent_addr
                .send(supervisor_request.supervisor_request_message)
                .await
                .unwrap()
                .unwrap();

            let supervisor_response = SupervisorResponse::new(supervisor_request.asset, response);
            
            SystemResponses::Supervisor(supervisor_response)

        }
        SystemMessages::Operational(operational_request) => {
            match operational_request.operational_target {
                OperationalTarget::Single(_id) => {
                    todo!();
                }
                OperationalTarget::All => {
                    let mut operational_infeasible_cases: Vec<OperationalResponseMessage> = vec![];
                    for (_asset, agent_registry) in &orchestrator.lock().unwrap().agent_registries {
                        for (_id, operational_addr) in &agent_registry.operational_agent_addrs {
                            operational_infeasible_cases.push(operational_addr.send(operational_request.operational_request_message.clone()).await.unwrap().unwrap());
                        }
                    }
                    let operational_response = OperationalResponse::new(OperationalTarget::All, operational_infeasible_cases);
                    SystemResponses::Operational(operational_response)
                }
            }
        }
        SystemMessages::Sap => {

            SystemResponses::Sap
        }
    };

    HttpResponse::Ok().json(system_responses)
}

impl Orchestrator {
    #[instrument(level = "info", skip_all)]
    async fn handle(&mut self, msg: OrchestratorRequest) -> Result<OrchestratorResponse, String> {
        match msg {
            OrchestratorRequest::SetWorkOrderState(work_order_number, status_codes) => {
                match self.scheduling_environment.lock().unwrap().work_orders_mut().inner.get_mut(&work_order_number) {
                    Some(work_order) => {
                        work_order.work_order_analytic.status_codes = status_codes;
                        work_order.initialize_weight();
                        let asset = work_order.functional_location().asset.clone();
                        let main_resource = work_order.main_work_center.clone();
                        
                        let update_work_order_message = UpdateWorkOrderMessage(work_order_number);
                        let actor_registry = self.agent_registries.get(&asset).unwrap();
                        actor_registry.strategic_agent_addr.do_send(update_work_order_message.clone());
                        actor_registry.tactical_agent_addr.do_send(update_work_order_message.clone());

                        actor_registry.supervisor_agent_addrs.iter().find(|id| id.0.2.as_ref().unwrap() == &main_resource).unwrap().1.do_send(update_work_order_message.clone());
                        for actor in actor_registry.operational_agent_addrs.values() {
                            actor.do_send(update_work_order_message.clone());
                        };
                        Ok(OrchestratorResponse::RequestStatus(format!("Status codes for {:?} updated correctly", work_order_number)))
                    }
                    None => Err(format!("Tried to update the status code for {:?}, but it was not found in the scheduling environment", work_order_number))
                }
            }
            OrchestratorRequest::AgentStatusRequest => {
                let _buffer = String::new();

                let mut agent_status_by_asset = HashMap::<Asset, AgentStatus>::new();
                for asset in self.agent_registries.keys() {
                    let strategic_agent_addr = self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .strategic_agent_addr
                        .clone();

                    let tactical_agent_addr = self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .tactical_agent_addr
                        .clone();

                    let strategic_agent_status = if let StrategicResponseMessage::Status(status) = strategic_agent_addr
                        .send(shared_messages::strategic::StrategicRequestMessage::Status(StrategicStatusMessage::General))
                        .await
                        .unwrap()
                        .unwrap() {
                        status
                    } else {
                        panic!()
                    };

                    let tactical_agent_status = if let TacticalResponseMessage::Status(status) = tactical_agent_addr
                        .send(TacticalRequestMessage::Status(TacticalStatusMessage::General))
                        .await
                        .unwrap()
                        .unwrap()
                        { 
                            status
                        } else {
                            panic!()
                        };

                    let mut supervisor_statai: Vec<SupervisorResponseStatus> = vec![];
                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .supervisor_agent_addrs
                        .iter()
                    {
                        let supervisor_agent_response =
                            addr.send(SupervisorRequestMessage::Status(SupervisorStatusMessage::General)).await.unwrap().unwrap();

                        let supervisor_agent_status = supervisor_agent_response.status();
                        supervisor_statai.push(supervisor_agent_status);
                    }

                    let mut operational_statai: Vec<OperationalStatusResponse> = vec![];
                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .operational_agent_addrs
                        .iter()
                    {
                        let operational_agent_response =
                            addr.send(OperationalRequestMessage::Status(OperationalStatusRequest::General)).await.unwrap().unwrap();

                        let _operational_agent_status = if let OperationalResponseMessage::Status(status) = operational_agent_response {
                            operational_statai.push(status) 
                        } else {
                            panic!()
                        };
                    }
                    let agent_status = AgentStatus::new(strategic_agent_status, tactical_agent_status, supervisor_statai, operational_statai);
                    agent_status_by_asset.insert(asset.clone(), agent_status);
                }
                let orchestrator_response_status = AgentStatusResponse::new(agent_status_by_asset);
                let orchestrator_response = OrchestratorResponse::AgentStatus(orchestrator_response_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::GetWorkOrderStatus(work_order_number, _level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: WorkOrders =
                    scheduling_environment_guard.clone_work_orders();

                let work_order_response:Option<(WorkOrderNumber, WorkOrderResponse)> = cloned_work_orders
                    .inner
                    .iter()
                    .find(|(work_order_number_key, _)| work_order_number == **work_order_number_key)
                    .map(|(work_order_number, work_order)| {
                        let work_order_response = WorkOrderResponse::new(
                            work_order.order_dates.earliest_allowed_start_period.clone(),
                            work_order.work_order_analytic.status_codes.awsc.clone(),
                            work_order.work_order_analytic.status_codes.sece.clone(),
                            work_order.work_order_info.revision.clone(),
                            work_order.work_order_info.work_order_type.clone(),
                            work_order.work_order_info.priority.clone(),
                            work_order.work_order_analytic.vendor.clone(),
                            work_order
                                .work_order_analytic
                                .status_codes
                                .material_status
                                .clone(),
                            work_order.work_order_analytic.work_order_weight,
                            work_order.work_order_info.unloading_point.clone(),
                            None,
                        );
                        (*work_order_number, work_order_response)
                    });

                let work_order_response = match work_order_response {
                    Some(response) => {
                        
                        let mut work_order_response = HashMap::new();
                        work_order_response.insert(response.0, response.1);
                        work_order_response
                    }
                    None => return Err(format!("{:?} was not found for the asset", work_order_number)),
                };
                
                let work_orders_status = WorkOrdersStatus::new(work_order_response);
                let orchestrator_response = OrchestratorResponse::WorkOrderStatus(work_orders_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::GetWorkOrdersState(asset, _level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: WorkOrders =
                    scheduling_environment_guard.clone_work_orders();
                let work_orders: WorkOrders = cloned_work_orders
                    .inner
                    .into_iter()
                    .filter(|wo| wo.1.work_order_info.functional_location.asset == asset)
                    .collect();

                let work_order_responses: HashMap<WorkOrderNumber, WorkOrderResponse> = work_orders
                    .inner
                    .iter()
                    .map(|(work_order_number, work_order)| {
                        let work_order_response = WorkOrderResponse::new(
                            work_order.order_dates.earliest_allowed_start_period.clone(),
                            work_order.work_order_analytic.status_codes.awsc.clone(),
                            work_order.work_order_analytic.status_codes.sece.clone(),
                            work_order.work_order_info.revision.clone(),
                            work_order.work_order_info.work_order_type.clone(),
                            work_order.work_order_info.priority.clone(),
                            work_order.work_order_analytic.vendor.clone(),
                            work_order
                                .work_order_analytic
                                .status_codes
                                .material_status
                                .clone(),
                            work_order.work_order_analytic.work_order_weight,
                            work_order.work_order_info.unloading_point.clone(),
                            None,
                        );
                        (*work_order_number, work_order_response)
                    })
                    .collect();

                let work_orders_status = WorkOrdersStatus::new(work_order_responses);

                let orchestrator_response = OrchestratorResponse::WorkOrderStatus(work_orders_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_strategic_periods();


                let strategic_periods = OrchestratorResponse::Periods(periods);
                Ok(strategic_periods)
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard.tactical_days();

                let tactical_days = OrchestratorResponse::Days(days.clone());
                Ok(tactical_days)
            }
            OrchestratorRequest::CreateSupervisorAgent(asset, id_string) => {
                let tactical_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .tactical_agent_addr
                    .clone();

                let supervisor_agent_addr = self.agent_factory.build_supervisor_agent(
                    asset.clone(),
                    id_string.clone(),
                    tactical_agent_addr,
                );

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_supervisor_agent(id_string.clone(), supervisor_agent_addr.clone());
                let response_string = format!("Supervisor agent created with id {}", id_string);
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
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

                let response_string = format!("Supervisor agent deleted with id {}", id);
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::CreateOperationalAgent(asset, id, operational_configuration) => {
                let supervisor_agent_addr = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_agent_addr_by_resource(&id.1[0].clone());
                
                let operational_agent_addr = self
                    .agent_factory
                    .build_operational_agent(id.clone(), operational_configuration, supervisor_agent_addr);

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_operational_agent(id.clone(), operational_agent_addr.clone());

                let response_string = format!("Operational agent created with id {}", id);
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
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

                let response_string = format!("Operational agent deleted  with id {}", id_string);
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::SetLogLevel(log_level) => {
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                let response_string = format!("Log level {}", log_level.to_level_string());
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)

            }
            OrchestratorRequest::SetProfiling(log_level) => {
                self.log_handles
                    .file_handle
                    .modify(|layer| {
                        *layer.filter_mut() = EnvFilter::new(log_level.to_level_string())
                    })
                    .unwrap();

                let response_string = format!("Profiling level {}", log_level.to_level_string());
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
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
                    .tactical_agent_addr
                    .send(shared_messages::SolutionExportMessage {})
                    .await;

                Ok(OrchestratorResponse::Export(format!(
                    "{{\"strategic_agent_solution\": {}, \"tactical_agent_solution\": {}}}",
                    strategic_agent_solution.unwrap(),
                    tactical_agent_solution.unwrap()
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use shared_messages::{models::{time_environment::day::Day, worker_environment::resources::Resources}, tactical::{Days, TacticalResources}};

    #[test]
    fn test_day_serialize() {

        let mut hash_map_nested = HashMap::<Day, f64>::new();
        
        let mut hash_map = HashMap::<Resources, Days>::new();
        let day = Day::new(0 ,Utc::now());
        day.to_string();
        hash_map_nested.insert(day, 123.0); 


        hash_map.insert(Resources::MtnMech, Days::new(hash_map_nested.clone()));
        let tactical_resources = TacticalResources::new(hash_map.clone());
        serde_json::to_string(&tactical_resources).unwrap();

    }

}
