use actix::prelude::*;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use shared_types::orchestrator::OrchestratorRequest;

use shared_types::orchestrator::WorkOrderResponse;
use shared_types::orchestrator::WorkOrdersStatus;
use shared_types::scheduling_environment::worker_environment::resources::Id;

use shared_types::scheduling_environment::worker_environment::WorkerEnvironment;
use shared_types::strategic::StrategicResponseMessage;
use shared_types::supervisor::SupervisorRequestMessage;
use shared_types::Asset;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use tracing::instrument;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::init::logging::LogHandles;
use shared_types::scheduling_environment::SchedulingEnvironment;

use shared_types::operational::operational_request_status::OperationalStatusRequest;
use shared_types::operational::operational_response_status::OperationalStatusResponse;
use shared_types::operational::{
    OperationalConfiguration, OperationalRequestMessage, OperationalResponseMessage,
};
use shared_types::orchestrator::{AgentStatus, AgentStatusResponse, OrchestratorResponse};
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_types::supervisor::supervisor_response_status::SupervisorResponseStatus;
use shared_types::supervisor::supervisor_status_message::SupervisorStatusMessage;

use shared_types::tactical::tactical_status_message::TacticalStatusMessage;
use shared_types::tactical::{TacticalRequestMessage, TacticalResponseMessage};
use tracing_subscriber::EnvFilter;

use shared_types::scheduling_environment::WorkOrders;

use super::AgentSpecific;
use super::ArcSwapSharedSolution;
use super::StateLink;

pub struct Orchestrator {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub arc_swap_shared_solutions: HashMap<Asset, Arc<ArcSwapSharedSolution>>,
    pub agent_factory: AgentFactory,
    pub agent_registries: HashMap<Asset, ActorRegistry>,
    pub agent_notify: Option<Weak<Mutex<Orchestrator>>>,
    pub log_handles: LogHandles,
}

// WARNING: Do not ever make this field public!
pub struct NotifyOrchestrator(Arc<Mutex<Orchestrator>>);

// WARNING: This should only take immutable references to self!
impl NotifyOrchestrator {
    pub fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()> {
        let locked_orchestrator = self.0.lock().unwrap();

        let agent_registry = locked_orchestrator
            .agent_registries
            .get(asset)
            .context("Asset should always be there")?;

        let state_link = StateLink::WorkOrders(AgentSpecific::Strategic(work_orders));

        agent_registry
            .strategic_agent_addr
            .do_send(state_link.clone());

        agent_registry
            .tactical_agent_addr
            .do_send(state_link.clone());

        agent_registry
            .supervisor_agent_addrs
            .values()
            .for_each(|addr| addr.do_send(state_link.clone()));

        agent_registry
            .operational_agent_addrs
            .values()
            .for_each(|addr| addr.do_send(state_link.clone()));

        Ok(())
    }
}

pub struct ActorRegistry {
    pub strategic_agent_addr: Addr<StrategicAgent>,
    pub tactical_agent_addr: Addr<TacticalAgent>,
    pub supervisor_agent_addrs: HashMap<Id, Addr<SupervisorAgent>>,
    pub operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
    pub number_of_operational_agents: Arc<AtomicU64>,
}

impl ActorRegistry {
    pub fn get_operational_addr(&self, operational_id: &String) -> Option<Addr<OperationalAgent>> {
        let option_id = self
            .operational_agent_addrs
            .iter()
            .find(|(id, _)| &id.0 == operational_id)
            .map(|(_, addr)| addr);
        option_id.cloned()
    }
}

impl Orchestrator {
    #[instrument(level = "info", skip_all)]
    pub async fn handle(
        &mut self,
        orchestrator_request: OrchestratorRequest,
    ) -> Result<OrchestratorResponse> {
        match orchestrator_request {
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

                    let strategic_agent_status = if let StrategicResponseMessage::Status(status) =
                        strategic_agent_addr
                            .send(shared_types::strategic::StrategicRequestMessage::Status(
                                StrategicStatusMessage::General,
                            ))
                            .await
                            .unwrap()
                            .unwrap()
                    {
                        status
                    } else {
                        panic!()
                    };

                    let tactical_agent_status = if let TacticalResponseMessage::Status(status) =
                        tactical_agent_addr
                            .send(TacticalRequestMessage::Status(
                                TacticalStatusMessage::General,
                            ))
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
                        let supervisor_agent_response = addr
                            .send(SupervisorRequestMessage::Status(
                                SupervisorStatusMessage::General,
                            ))
                            .await
                            .unwrap()
                            .unwrap();

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
                        let operational_agent_response = addr
                            .send(OperationalRequestMessage::Status(
                                OperationalStatusRequest::General,
                            ))
                            .await
                            .unwrap()
                            .unwrap();

                        if let OperationalResponseMessage::Status(status) =
                            operational_agent_response
                        {
                            operational_statai.push(status)
                        } else {
                            panic!()
                        };
                    }
                    let agent_status = AgentStatus::new(
                        strategic_agent_status,
                        tactical_agent_status,
                        supervisor_statai,
                        operational_statai,
                    );
                    agent_status_by_asset.insert(asset.clone(), agent_status);
                }
                let orchestrator_response_status = AgentStatusResponse::new(agent_status_by_asset);
                let orchestrator_response =
                    OrchestratorResponse::AgentStatus(orchestrator_response_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::InitializeSystemAgentsFromFile(asset, system_agents) => {
                // FIX TODO: send message to the strategic agent to update its resources.
                {
                    let mut scheduling_environment_guard =
                        self.scheduling_environment.lock().unwrap();
                    scheduling_environment_guard
                        .worker_environment
                        .system_agents = system_agents;
                }

                let state_link = StateLink::WorkerEnvironment;
                let agent_registry = self.agent_registries.get(&asset).unwrap();
                agent_registry
                    .strategic_agent_addr
                    .do_send(state_link.clone());

                agent_registry
                    .tactical_agent_addr
                    .do_send(state_link.clone());

                for supervisor in agent_registry.supervisor_agent_addrs.iter() {
                    supervisor.1.do_send(state_link.clone());
                }

                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let operational_agents = &scheduling_environment_guard
                    .worker_environment
                    .system_agents
                    .operational;

                for agent in operational_agents {
                    let id = Id::new(agent.id.clone(), agent.resources.resources.clone(), None);
                    let supervisor_addrs = self
                        .agent_registries
                        .get(&asset)
                        .with_context(|| {
                            format!(
                                "{:#?} not found for {:#?}",
                                std::any::type_name::<ActorRegistry>(),
                                &asset
                            )
                        })?
                        .supervisor_agent_addrs
                        .clone();
                    let arc_swap =
                        self.arc_swap_shared_solutions
                            .get(&asset)
                            .with_context(|| {
                                format!(
                                    "{:#?} not found for {:#?}",
                                    std::any::type_name::<ArcSwapSharedSolution>(),
                                    &asset
                                )
                            })?;
                    let notify_orchestrator = NotifyOrchestrator(
                        self.agent_notify
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .with_context(|| {
                                format!(
                                    "{:?} could not be upgraded to {:?}",
                                    std::any::type_name::<Weak<Mutex<Orchestrator>>>(),
                                    std::any::type_name::<Arc<Mutex<Orchestrator>>>()
                                )
                            })?,
                    );
                    let operational_addr = self.agent_factory.build_operational_agent(
                        id.clone(),
                        &agent.operational_configuration,
                        supervisor_addrs,
                        Arc::clone(arc_swap),
                        notify_orchestrator,
                    );

                    self.agent_registries
                        .get_mut(&asset)
                        .unwrap()
                        .operational_agent_addrs
                        .insert(id, operational_addr);
                }
                Ok(OrchestratorResponse::Success)
            }
            OrchestratorRequest::GetWorkOrderStatus(work_order_number, _level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: WorkOrders =
                    scheduling_environment_guard.work_orders.clone();

                let (_, work_order) = cloned_work_orders
                    .inner
                    .iter()
                    .find(|(won, _)| work_order_number == **won)
                    .with_context(|| {
                        format!(
                            "{:?} is not part of the SchedulingEnvironment",
                            work_order_number
                        )
                    })?;

                let asset = &work_order.work_order_info.functional_location.asset;

                let api_solution = match self.arc_swap_shared_solutions.get(asset) {
                    Some(arc_swap_shared_solution) => (arc_swap_shared_solution).0.load(),
                    None => bail!("Asset: {:?} is not initialzed", &asset),
                };

                let work_order_response =
                    WorkOrderResponse::new(work_order, (**api_solution).clone().into());

                let work_orders_status = WorkOrdersStatus::Single(work_order_response);

                let orchestrator_response =
                    OrchestratorResponse::WorkOrderStatus(work_orders_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::GetWorkOrdersState(asset, _level_of_detail) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: WorkOrders =
                    scheduling_environment_guard.work_orders.clone();
                let work_orders: WorkOrders = cloned_work_orders
                    .inner
                    .into_iter()
                    .filter(|wo| wo.1.work_order_info.functional_location.asset == asset)
                    .collect();

                let loaded_shared_solution = match self.arc_swap_shared_solutions.get(&asset) {
                    Some(arc_swap_shared_solution) => arc_swap_shared_solution.0.load(),
                    None => bail!("Ordinator has not been initialized for asset: {}", &asset),
                };
                let work_order_responses: HashMap<WorkOrderNumber, WorkOrderResponse> = work_orders
                    .inner
                    .iter()
                    .map(|(work_order_number, work_order)| {
                        let work_order_response = WorkOrderResponse::new(
                            work_order,
                            (**loaded_shared_solution).clone().into(),
                        );
                        (*work_order_number, work_order_response)
                    })
                    .collect();

                let work_orders_status = WorkOrdersStatus::Multiple(work_order_responses);

                let orchestrator_response =
                    OrchestratorResponse::WorkOrderStatus(work_orders_status);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard
                    .time_environment
                    .strategic_periods()
                    .clone();

                let strategic_periods = OrchestratorResponse::Periods(periods);
                Ok(strategic_periods)
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard
                    .time_environment
                    .tactical_days();

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

                let number_of_operational_agents = Arc::clone(
                    &self
                        .agent_registries
                        .get(&asset)
                        .unwrap()
                        .number_of_operational_agents,
                );
                let supervisor_agent_addr = self.agent_factory.build_supervisor_agent(
                    asset.clone(),
                    id_string.clone(),
                    tactical_agent_addr,
                    self.arc_swap_shared_solutions.get(&asset).unwrap().clone(),
                    number_of_operational_agents,
                    NotifyOrchestrator(
                        self.agent_notify
                            .as_ref()
                            .expect("Orchestrator is initialized with the Option::Some variant")
                            .upgrade()
                            .expect("This Weak reference should always be able to be upgraded."),
                    ),
                );

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_supervisor_agent(
                        id_string.clone(),
                        supervisor_agent_addr
                            .expect("Could not create SupervisorAgent")
                            .clone(),
                    );
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
                let response_string = format!("Operational agent created with id {}", id);

                self.create_operational_agent(&asset, id, &operational_configuration);

                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);

                Ok(orchestrator_response)
            }
            OrchestratorRequest::DeleteOperationalAgent(asset, id_string) => {
                let id = self
                    .agent_registries
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string.clone());

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
            OrchestratorRequest::Export(_asset) => {
                panic!();
            }
        }
    }

    pub fn initialize_operational_agents(&mut self, asset: Asset) {
        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
        let worker_environment = &scheduling_environment_guard
            .worker_environment
            .system_agents
            .operational
            .clone();
        drop(scheduling_environment_guard);

        for operational_agent in worker_environment.iter() {
            let id: Id = Id::new(
                operational_agent.id.clone(),
                operational_agent.resources.resources.clone(),
                None,
            );
            self.create_operational_agent(&asset, id, &operational_agent.operational_configuration);
        }
    }

    fn create_operational_agent(
        &mut self,
        asset: &Asset,
        id: Id,
        operational_configuration: &OperationalConfiguration,
    ) {
        let operational_agent_addr = self.agent_factory.build_operational_agent(
            id.clone(),
            operational_configuration,
            self.agent_registries
                .get(asset)
                .unwrap()
                .supervisor_agent_addrs
                .clone(),
            self.arc_swap_shared_solutions.get(asset).unwrap().clone(),
            NotifyOrchestrator(
                self.agent_notify
                    .as_ref()
                    .expect("Orchestrator is initialized with the Option::Some variant")
                    .upgrade()
                    .expect("This Weak reference should always be able to be updated"),
            ),
        );

        self.agent_registries
            .get(asset)
            .unwrap()
            .number_of_operational_agents
            .fetch_add(1, Ordering::SeqCst);

        self.agent_registries
            .get_mut(asset)
            .unwrap()
            .add_operational_agent(id.clone(), operational_agent_addr.clone());
    }
}

impl ActorRegistry {
    fn new(
        strategic_agent_addr: Addr<StrategicAgent>,
        tactical_agent_addr: Addr<TacticalAgent>,
        supervisor_agent_addrs: HashMap<Id, Addr<SupervisorAgent>>,
        number_of_operational_agents: Arc<AtomicU64>,
    ) -> Self {
        ActorRegistry {
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_agent_addrs,
            operational_agent_addrs: HashMap::new(),
            number_of_operational_agents,
        }
    }

    pub fn add_supervisor_agent(&mut self, id: Id, addr: Addr<SupervisorAgent>) {
        self.supervisor_agent_addrs.insert(id, addr);
    }

    pub fn add_operational_agent(&mut self, id: Id, addr: Addr<OperationalAgent>) {
        self.operational_agent_addrs.insert(id, addr);
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> Id {
        self.supervisor_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

impl Orchestrator {
    pub async fn new_with_arc(
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        log_handles: LogHandles,
    ) -> Arc<Mutex<Self>> {
        let agent_factory = agent_factory::AgentFactory::new(scheduling_environment.clone());

        let orchestrator = Orchestrator {
            scheduling_environment,
            arc_swap_shared_solutions: HashMap::new(),
            agent_factory,
            agent_registries: HashMap::new(),
            log_handles,
            agent_notify: None,
        };

        let arc_orchestrator = Arc::new(Mutex::new(orchestrator));

        arc_orchestrator.lock().unwrap().agent_notify = Some(Arc::downgrade(&arc_orchestrator));
        arc_orchestrator
    }

    // How should this asset function be implemented. The real question is what should be done about the
    // the files versus bitstream. One thing is for sure if the add_asset function should be reused
    // there can be no file handling inside of it.
    pub fn add_asset(&mut self, asset: Asset, system_agents_bytes: Vec<u8>) -> Result<()> {
        let mut scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

        scheduling_environment_guard
            .worker_environment
            .initialize_from_resource_configuration_file(system_agents_bytes)
            .with_context(|| {
                format!(
                    "{} not correctly parsed",
                    std::any::type_name::<WorkerEnvironment>().bright_red()
                )
            })?;

        let shared_solutions_arc_swap = AgentFactory::create_shared_solution_arc_swap();

        let strategic_agent_addr = self
            .agent_factory
            .build_strategic_agent(
                asset.clone(),
                &scheduling_environment_guard,
                shared_solutions_arc_swap.clone(),
                NotifyOrchestrator(
                    self.agent_notify
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .expect("Weak reference part of initialization"),
                ),
            )
            .context("Could not build the StrategicAgent")?;

        let tactical_agent_addr = self.agent_factory.build_tactical_agent(
            asset.clone(),
            strategic_agent_addr.clone(),
            &scheduling_environment_guard,
            shared_solutions_arc_swap.clone(),
            NotifyOrchestrator(
                self.agent_notify
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .expect("Weak reference part of initialization"),
            ),
        );

        let supervisors = scheduling_environment_guard
            .worker_environment
            .system_agents
            .supervisors
            .clone();
        drop(scheduling_environment_guard);
        let mut supervisor_addrs = HashMap::<Id, Addr<SupervisorAgent>>::new();
        let number_of_operational_agents = Arc::new(AtomicU64::new(0));

        for supervisor in supervisors {
            let id = Id::new("default".to_string(), vec![], Some(supervisor.clone()));

            let supervisor_addr = self
                .agent_factory
                .build_supervisor_agent(
                    asset.clone(),
                    id.clone(),
                    tactical_agent_addr.clone(),
                    shared_solutions_arc_swap.clone(),
                    Arc::clone(&number_of_operational_agents),
                    NotifyOrchestrator(
                        self.agent_notify
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .expect("Weak reference part of initialization"),
                    ),
                )
                .expect("AgentFactory could not build the specified supervisor agent");

            supervisor_addrs.insert(id, supervisor_addr);
        }

        let agent_registry = ActorRegistry::new(
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_addrs,
            number_of_operational_agents,
        );
        self.arc_swap_shared_solutions
            .insert(asset.clone(), shared_solutions_arc_swap);

        self.agent_registries.insert(asset, agent_registry);
        Ok(())
    }
}
