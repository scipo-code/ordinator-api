use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use shared_types::orchestrator::OrchestratorRequest;

use shared_types::orchestrator::WorkOrderResponse;
use shared_types::orchestrator::WorkOrdersStatus;
use shared_types::scheduling_environment::worker_environment::resources::Id;

use shared_types::scheduling_environment::worker_environment::WorkerEnvironment;
use shared_types::strategic::StrategicRequestMessage;
use shared_types::strategic::StrategicResponseMessage;
use shared_types::supervisor::SupervisorRequestMessage;
use shared_types::supervisor::SupervisorResponseMessage;
use shared_types::Asset;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use tracing::instrument;

use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::init::logging::LogHandles;
use shared_types::scheduling_environment::SchedulingEnvironment;

use shared_types::operational::operational_request_status::OperationalStatusRequest;
use shared_types::operational::operational_response_status::OperationalResponseStatus;
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

use super::AgentMessage;
use super::AgentSpecific;
use super::ArcSwapSharedSolution;
use super::StateLink;

pub struct Orchestrator {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub arc_swap_shared_solutions: HashMap<Asset, Arc<ArcSwapSharedSolution>>,
    pub agent_factory: AgentFactory,
    pub agent_registries: HashMap<Asset, AgentRegistry>,
    pub agent_notify: Option<Weak<Mutex<Orchestrator>>>,
    pub log_handles: LogHandles,
}

// WARNING: Do not ever make this field public!
#[derive(Clone)]
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

        let state_link = AgentMessage::State(StateLink::WorkOrders(AgentSpecific::Strategic(
            work_orders.clone(),
        )));

        agent_registry
            .strategic_agent_sender
            .sender
            .send(state_link)?;

        let state_link = AgentMessage::State(StateLink::WorkOrders(AgentSpecific::Strategic(
            work_orders.clone(),
        )));

        agent_registry
            .tactical_agent_sender
            .sender
            .send(state_link)?;

        for comm in agent_registry.supervisor_agent_senders.values() {
            let state_link = AgentMessage::State(StateLink::WorkOrders(AgentSpecific::Strategic(
                work_orders.clone(),
            )));
            comm.sender.send(state_link)?;
        }

        for addr in agent_registry.operational_agent_senders.values() {
            let state_link = AgentMessage::State(StateLink::WorkOrders(AgentSpecific::Strategic(
                work_orders.clone(),
            )));
            addr.sender.send(state_link)?;
        }

        Ok(())
    }
}

pub struct AgentRegistry {
    pub strategic_agent_sender:
        Communication<AgentMessage<StrategicRequestMessage>, StrategicResponseMessage>,
    pub tactical_agent_sender:
        Communication<AgentMessage<TacticalRequestMessage>, TacticalResponseMessage>,
    pub supervisor_agent_senders: HashMap<
        Id,
        Communication<AgentMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
    >,
    pub operational_agent_senders: HashMap<
        Id,
        Communication<AgentMessage<OperationalRequestMessage>, OperationalResponseMessage>,
    >,
    pub number_of_operational_agents: Arc<AtomicU64>,
}

pub struct Communication<Req, Res> {
    pub sender: Sender<Req>,
    pub receiver: Receiver<Result<Res>>,
}

impl AgentRegistry {
    pub fn get_operational_addr(
        &self,
        operational_id: &String,
    ) -> Option<&Communication<AgentMessage<OperationalRequestMessage>, OperationalResponseMessage>>
    {
        let option_id = self
            .operational_agent_senders
            .iter()
            .find(|(id, _)| &id.0 == operational_id)
            .map(|(_, addr)| addr);
        option_id
    }

    // This function should be generic over all the different types of messages.
    // So the idea behind this function is that it should take a generic for
    // the interal message, but that the outer message is the same for every
    // agent! This means that it should take like `Status` or something like
    // that
    // FIX
    // Make this generic
    // WARN
    // Making this generic is probably not the best idea.
    pub fn recv_all_agents_status(&self) -> Result<AgentStatus> {
        let mut supervisor_statai: Vec<SupervisorResponseStatus> = vec![];
        let mut operational_statai: Vec<OperationalResponseStatus> = vec![];

        let strategic = self.strategic_agent_sender.receiver.recv()??;

        let strategic_status = if let StrategicResponseMessage::Status(strategic) = strategic {
            strategic
        } else {
            panic!()
        };

        let tactical = self.tactical_agent_sender.receiver.recv()??;
        let tactical_status = if let TacticalResponseMessage::Status(tactical) = tactical {
            tactical
        } else {
            panic!()
        };

        for receiver in self.supervisor_agent_senders.iter() {
            let supervisor = receiver.1.receiver.recv()??;
            if let SupervisorResponseMessage::Status(supervisor) = supervisor {
                supervisor_statai.push(supervisor);
            } else {
                panic!()
            }
        }
        for receiver in self.operational_agent_senders.iter() {
            let operational = receiver.1.receiver.recv()??;

            if let OperationalResponseMessage::Status(operational) = operational {
                operational_statai.push(operational);
            } else {
                panic!()
            }
        }

        let agent_status = AgentStatus {
            strategic_status,
            tactical_status,
            supervisor_statai,
            operational_statai,
        };
        Ok(agent_status)
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
                    let strategic_agent_addr = &self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .strategic_agent_sender;

                    let tactical_agent_addr = &self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .tactical_agent_sender;

                    // What should we do here? I think that the best approach will be to make the code function
                    strategic_agent_addr.sender.send(AgentMessage::Actor(
                        shared_types::strategic::StrategicRequestMessage::Status(
                            StrategicStatusMessage::General,
                        ),
                    ))?;

                    tactical_agent_addr.sender.send(AgentMessage::Actor(
                        TacticalRequestMessage::Status(TacticalStatusMessage::General),
                    ))?;

                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .supervisor_agent_senders
                        .iter()
                    {
                        addr.sender
                            .send(AgentMessage::Actor(SupervisorRequestMessage::Status(
                                SupervisorStatusMessage::General,
                            )))?
                    }

                    for (_id, addr) in self
                        .agent_registries
                        .get(asset)
                        .unwrap()
                        .operational_agent_senders
                        .iter()
                    {
                        addr.sender.send(AgentMessage::Actor(
                            OperationalRequestMessage::Status(OperationalStatusRequest::General),
                        ))?;
                    }

                    let agent_status = self
                        .agent_registries
                        .get(asset)
                        .expect("Asset should always be present")
                        .recv_all_agents_status()?;

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

                let state_link = AgentMessage::State(StateLink::WorkerEnvironment);
                let agent_registry = self.agent_registries.get(&asset).unwrap();
                agent_registry
                    .strategic_agent_sender
                    .sender
                    .send(state_link)?;

                let state_link = AgentMessage::State(StateLink::WorkerEnvironment);
                agent_registry
                    .tactical_agent_sender
                    .sender
                    .send(state_link)?;

                for supervisor in agent_registry.supervisor_agent_senders.iter() {
                    let state_link = AgentMessage::State(StateLink::WorkerEnvironment);
                    supervisor.1.sender.send(state_link)?;
                }

                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let operational_agents = &scheduling_environment_guard
                    .worker_environment
                    .system_agents
                    .operational;

                for agent in operational_agents {
                    let id = Id::new(agent.id.clone(), agent.resources.resources.clone(), None);
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
                    let operational_communication = self
                        .agent_factory
                        .build_operational_agent(
                            &asset,
                            &id,
                            &agent.operational_configuration,
                            Arc::clone(arc_swap),
                            notify_orchestrator,
                        )
                        .context("Could not build OperationalAgent")?;

                    self.agent_registries
                        .get_mut(&asset)
                        .unwrap()
                        .operational_agent_senders
                        .insert(id, operational_communication);
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
                let notify_orchestrator = NotifyOrchestrator(
                    self.agent_notify
                        .as_ref()
                        .expect("Orchestrator is initialized with the Option::Some variant")
                        .upgrade()
                        .expect("This Weak reference should always be able to be upgraded."),
                );

                let supervisor_agent_addr = self.agent_factory.build_supervisor_agent(
                    &asset,
                    &id_string,
                    self.arc_swap_shared_solutions.get(&asset).unwrap().clone(),
                    notify_orchestrator,
                );

                self.agent_registries
                    .get_mut(&asset)
                    .unwrap()
                    .add_supervisor_agent(
                        id_string.clone(),
                        supervisor_agent_addr.expect("Could not create SupervisorAgent"),
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
                    .supervisor_agent_senders
                    .remove(&id);

                let response_string = format!("Supervisor agent deleted with id {}", id);
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::CreateOperationalAgent(asset, id, operational_configuration) => {
                let response_string = format!("Operational agent created with id {}", id);

                self.create_operational_agent(&asset, &id, &operational_configuration)?;

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
                    .operational_agent_senders
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

    pub fn initialize_operational_agents(&mut self, asset: Asset) -> Result<()> {
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
            self.create_operational_agent(
                &asset,
                &id,
                &operational_agent.operational_configuration,
            )?;
        }
        Ok(())
    }

    fn create_operational_agent(
        &mut self,
        asset: &Asset,
        id: &Id,
        operational_configuration: &OperationalConfiguration,
    ) -> Result<()> {
        let operational_agent_addr = self.agent_factory.build_operational_agent(
            asset,
            id,
            operational_configuration,
            self.arc_swap_shared_solutions.get(asset).unwrap().clone(),
            NotifyOrchestrator(
                self.agent_notify
                    .as_ref()
                    .expect("Orchestrator is initialized with the Option::Some variant")
                    .upgrade()
                    .expect("This Weak reference should always be able to be updated"),
            ),
        )?;

        self.agent_registries
            .get(asset)
            .unwrap()
            .number_of_operational_agents
            .fetch_add(1, Ordering::SeqCst);

        self.agent_registries
            .get_mut(asset)
            .unwrap()
            .add_operational_agent(id.clone(), operational_agent_addr);
        Ok(())
    }
}

impl AgentRegistry {
    fn new(
        strategic_agent_addr: Communication<
            AgentMessage<StrategicRequestMessage>,
            StrategicResponseMessage,
        >,
        tactical_agent_addr: Communication<
            AgentMessage<TacticalRequestMessage>,
            TacticalResponseMessage,
        >,
        supervisor_agent_addrs: HashMap<
            Id,
            Communication<AgentMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
        >,
        number_of_operational_agents: Arc<AtomicU64>,
    ) -> Self {
        AgentRegistry {
            strategic_agent_sender: strategic_agent_addr,
            tactical_agent_sender: tactical_agent_addr,
            supervisor_agent_senders: supervisor_agent_addrs,
            operational_agent_senders: HashMap::new(),
            number_of_operational_agents,
        }
    }

    pub fn add_supervisor_agent(
        &mut self,
        id: Id,
        communication: Communication<
            AgentMessage<SupervisorRequestMessage>,
            SupervisorResponseMessage,
        >,
    ) {
        self.supervisor_agent_senders.insert(id, communication);
    }

    pub fn add_operational_agent(
        &mut self,
        id: Id,
        communication: Communication<
            AgentMessage<OperationalRequestMessage>,
            OperationalResponseMessage,
        >,
    ) {
        self.operational_agent_senders.insert(id, communication);
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> Id {
        self.supervisor_agent_senders
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

        let notify_orchestrator = NotifyOrchestrator(
            self.agent_notify
                .as_ref()
                .unwrap()
                .upgrade()
                .expect("Weak reference part of initialization"),
        );

        let tactical_agent_addr = self
            .agent_factory
            .build_tactical_agent(
                &asset,
                &scheduling_environment_guard,
                shared_solutions_arc_swap.clone(),
                notify_orchestrator.clone(),
            )
            .context("Could not build TacticalAgent")?;

        let supervisors = scheduling_environment_guard
            .worker_environment
            .system_agents
            .supervisors
            .clone();
        drop(scheduling_environment_guard);
        let mut supervisor_addrs = HashMap::<
            Id,
            Communication<AgentMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
        >::new();
        let number_of_operational_agents = Arc::new(AtomicU64::new(0));

        for supervisor in supervisors {
            let id = Id::new("default".to_string(), vec![], Some(supervisor.clone()));

            let supervisor_addr = self
                .agent_factory
                .build_supervisor_agent(
                    &asset,
                    &id,
                    shared_solutions_arc_swap.clone(),
                    notify_orchestrator.clone(),
                )
                .expect("AgentFactory could not build the specified supervisor agent");

            supervisor_addrs.insert(id, supervisor_addr);
        }

        let agent_registry = AgentRegistry::new(
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
