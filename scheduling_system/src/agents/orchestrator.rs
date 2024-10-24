use actix::prelude::*;
use shared_types::orchestrator::OrchestratorMessage;
use shared_types::orchestrator::OrchestratorRequest;

use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources;
use shared_types::scheduling_environment::worker_environment::resources::Id;

use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::Periods;
use shared_types::strategic::StrategicResources;
use shared_types::strategic::StrategicResponseMessage;
use shared_types::supervisor::SupervisorRequestMessage;
use shared_types::tactical::Days;
use shared_types::tactical::TacticalResources;
use shared_types::Asset;
use shared_types::TomlAgents;
use tracing::instrument;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use strum::IntoEnumIterator;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::init::logging::LogHandles;
use shared_types::scheduling_environment::SchedulingEnvironment;

use dotenvy::dotenv;
use shared_types::operational::operational_request_status::OperationalStatusRequest;
use shared_types::operational::operational_response_status::OperationalStatusResponse;
use shared_types::operational::{
    OperationalConfiguration, OperationalRequestMessage,
    OperationalResponseMessage,
};
use shared_types::orchestrator::{AgentStatus, AgentStatusResponse, OrchestratorResponse};
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_types::strategic::strategic_response_status::{WorkOrderResponse, WorkOrdersStatus};
use shared_types::supervisor::supervisor_response_status::SupervisorResponseStatus;
use shared_types::supervisor::supervisor_status_message::SupervisorStatusMessage;

use shared_types::tactical::tactical_status_message::TacticalStatusMessage;
use shared_types::tactical::{
    TacticalRequestMessage,  TacticalResponseMessage,
};
use tracing_subscriber::EnvFilter;

use crate::agents::UpdateWorkOrderMessage;
use shared_types::scheduling_environment::WorkOrders;

#[derive(Clone, Debug)]
pub struct ArcOrchestrator(pub Arc<Mutex<Orchestrator>>);

#[derive(Debug, Clone)]
pub struct Orchestrator {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub agent_factory: AgentFactory,
    pub agent_registries: HashMap<Asset, ActorRegistry>,
    pub log_handles: LogHandles,
}

#[derive(Clone, Debug)]
pub struct ActorRegistry {
    pub strategic_agent_addr: Addr<StrategicAgent>,
    pub tactical_agent_addr: Addr<TacticalAgent>,
    pub supervisor_agent_addrs: HashMap<Id, Addr<SupervisorAgent>>,
    pub operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
    pub number_of_operational_agents: Arc<AtomicU64>
}

impl ActorRegistry {
    pub fn get_operational_addr(&self, operational_id: &String) -> Option<Addr<OperationalAgent>> {
        let option_id = self.operational_agent_addrs.iter().find(|(id, _)| {
            &id.0 == operational_id
        }).map(|(_, addr)| addr);
        option_id.cloned()
    }
}

impl Orchestrator {
    #[instrument(level = "info", skip_all)]
    pub async fn handle(&mut self, orchestrator_request: OrchestratorRequest) -> Result<OrchestratorResponse, String> {
        match orchestrator_request {
            OrchestratorRequest::SetWorkOrderState(work_order_number, status_codes) => {
                match self.scheduling_environment.lock().unwrap().work_orders_mut().inner.get_mut(&work_order_number) {
                    Some(work_order) => {
                        work_order.work_order_analytic.status_codes = status_codes;
                        work_order.initialize_weight();
                        let asset = work_order.functional_location().asset.clone();
                        
                        let update_work_order_message = UpdateWorkOrderMessage(work_order_number);
                        let actor_registry = self.agent_registries.get(&asset).unwrap();
                        actor_registry.strategic_agent_addr.do_send(update_work_order_message.clone());
                        actor_registry.tactical_agent_addr.do_send(update_work_order_message.clone());

                        panic!("Fix the bug below");
                        // TODO actor_registry.supervisor_agent_addrs.iter().find(|id| id.0.2.as_ref().unwrap() == &main_resource).unwrap().1.do_send(update_work_order_message.clone());
                        // for actor in actor_registry.operational_agent_addrs.values() {
                        //     actor.do_send(update_work_order_message.clone());
                        // };
                        // Ok(OrchestratorResponse::RequestStatus(format!("Status codes for {:?} updated correctly", work_order_number)))
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
                        .send(shared_types::strategic::StrategicRequestMessage::Status(StrategicStatusMessage::General))
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

                        if let OperationalResponseMessage::Status(status) = operational_agent_response {
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
                            work_order.work_order_dates.earliest_allowed_start_period.clone(),
                            work_order.work_order_info.clone(),
                            work_order.work_order_analytic.vendor,
                            work_order.work_order_analytic.work_order_weight,
                            work_order.work_order_analytic.status_codes.clone(),
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
                            work_order.work_order_dates.earliest_allowed_start_period.clone(),
                            work_order.work_order_info.clone(),
                            work_order.work_order_analytic.vendor,
                            work_order.work_order_analytic.work_order_weight,
                            work_order.work_order_analytic.status_codes.clone(),
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

                let number_of_operational_agents = Arc::clone(&self.agent_registries.get(&asset).unwrap().number_of_operational_agents);
                let supervisor_agent_addr = self.agent_factory.build_supervisor_agent(
                    asset.clone(),
                    id_string.clone(),
                    tactical_agent_addr,
                    number_of_operational_agents,


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

                supervisor_agent_addr.do_send(shared_types::StopMessage {});

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

                self.create_operational_agent(&asset, id, operational_configuration);

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

                operational_agent_addr.do_send(shared_types::StopMessage {});

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

    pub fn initialize_agents_from_env(&mut self, asset: Asset) {
        dotenv().expect("Could not load in the .env file.");
        
        let asset_string = dotenvy::var("ASSET").expect("ASSET environment variable should always be set");

        let resource_string = format!("./configuration/resources_{}.toml", asset_string.to_lowercase());


        let toml_agents_string: String = std::fs::read_to_string(resource_string).unwrap(); 
        let toml_agents: TomlAgents = toml::from_str(&toml_agents_string).unwrap();

        for agent in toml_agents.operational {
            let id: Id = Id::new(agent.id, agent.resources.resources, None);
            self.create_operational_agent(&asset, id, agent.operational_configuration.into());
        }

        
    }

    fn create_operational_agent(&mut self, asset: &Asset, id: Id, operational_configuration: OperationalConfiguration) {
    
        let (operational_objective, operational_agent_addr) = self
            .agent_factory
            .build_operational_agent(id.clone(), operational_configuration, self.agent_registries.get(asset).unwrap().supervisor_agent_addrs.clone());

        let operational_id_and_objective = OrchestratorMessage::new((id.clone(), operational_objective));


        self.agent_registries.get(&asset).unwrap().supervisor_agent_addrs.iter().for_each(|(_, sup_addr)| {
            sup_addr.do_send(operational_id_and_objective.clone())
        });

        self.agent_registries.get(&asset).unwrap().number_of_operational_agents.fetch_add(1, Ordering::SeqCst);

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

    pub fn supervisor_agent_addr(&self, id: Id) -> Addr<SupervisorAgent> {
        self.supervisor_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn operational_agent_addr(&self, id: Id) -> Addr<OperationalAgent> {
        self.operational_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> Id {
        self.supervisor_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }

    #[allow(dead_code)]
    pub fn operational_by_id_string(&self, id_string: String) -> Id {
        self.operational_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

impl Orchestrator {
    pub fn new(
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        log_handles: LogHandles,
    ) -> Self {
        let agent_factory = agent_factory::AgentFactory::new(scheduling_environment.clone());

        let agent_registries = HashMap::new();

        Orchestrator {
            scheduling_environment,
            agent_factory,
            agent_registries,
            log_handles,
        }
    }

    pub fn add_asset(&mut self, asset: Asset) {

        let asset_string = dotenvy::var("ASSET").expect("ASSET environment variable should always be set");

        let resource_string = format!("./configuration/resources_{}.toml", asset_string.to_lowercase());

        let toml_agents_path = Path::new(&resource_string);

        let strategic_resources = self.generate_strategic_resources(toml_agents_path);

        let tactical_resources = self.generate_tactical_resources(toml_agents_path);

        let strategic_agent_addr = self
            .agent_factory
            .build_strategic_agent(asset.clone(), Some(strategic_resources));

        let tactical_agent_addr = self.agent_factory.build_tactical_agent(
            asset.clone(),
            strategic_agent_addr.clone(),
            Some(tactical_resources),
        );

        let mut supervisor_addrs = HashMap::<Id, Addr<SupervisorAgent>>::new();
        let number_of_operational_agents = Arc::new(AtomicU64::new(0));

        let resources_config_string = std::fs::read_to_string(toml_agents_path).unwrap();

        let resources_config: TomlAgents = toml::from_str(&resources_config_string).unwrap();
        for supervisor in resources_config.supervisors {
            let id = Id::new("default".to_string(), vec![], Some(supervisor));

            let supervisor_addr = self.agent_factory.build_supervisor_agent(
                asset.clone(),
                id.clone(),
                tactical_agent_addr.clone(),
                Arc::clone(&number_of_operational_agents),
            );

            supervisor_addrs.insert(id, supervisor_addr);
        }

        let agent_registry =
            ActorRegistry::new(strategic_agent_addr, tactical_agent_addr, supervisor_addrs, number_of_operational_agents);

        self.agent_registries.insert(asset, agent_registry);
    }

    fn generate_strategic_resources(&self, toml_agents_path: &Path) -> StrategicResources {
        let periods: Vec<Period> = self
            .scheduling_environment
            .lock()
            .unwrap()
            .periods()
            .clone();

        let contents = std::fs::read_to_string(toml_agents_path).unwrap();

        let config: TomlAgents = toml::from_str(&contents).unwrap();

        let _hours_per_day = 6.0;
        let days_in_period = 13.0;

        let gradual_reduction = |i: usize| -> f64 {
            if i == 0 {
                1.0
            } else if i == 1 {
                0.9
            } else if i == 2 {
                0.8
            } else {
                0.6
            }
        };

        let mut resources_hash_map = HashMap::<Resources, Periods>::new();
        for operational_agent in config.operational {
            for (i, period) in periods.clone().iter().enumerate() {
                let resource_periods = resources_hash_map
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Periods(HashMap::new()));

                *resource_periods
                    .0
                    .entry(period.clone())
                    .or_insert_with(|| Work::from(0.0)) +=
                    Work::from(operational_agent.hours_per_day * days_in_period * gradual_reduction(i))
            }
        }

        StrategicResources::new(resources_hash_map)
    }
    fn generate_tactical_resources(&self, toml_path: &Path) -> TacticalResources {
        let days: Vec<Day> = self
            .scheduling_environment
            .lock()
            .unwrap()
            .tactical_days()
            .clone();

        let contents = std::fs::read_to_string(toml_path).unwrap();

        let config: TomlAgents = toml::from_str(&contents).unwrap();

        let _hours_per_day = 6.0;

        let gradual_reduction = |i: usize| -> f64 {
            match i {
                0..=13 => 1.0,
                14..=27 => 1.0,
                _ => 1.0,
            }
        };

        let mut resources_hash_map = HashMap::<Resources, Days>::new();
        for operational_agent in config.operational {
            for (i, day) in days.clone().iter().enumerate() {
                let resource_periods = resources_hash_map
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Days::new(HashMap::new()));

                *resource_periods
                    .days
                    .entry(day.clone())
                    .or_insert_with(|| Work::from(0.0)) +=
                    Work::from(operational_agent.hours_per_day * gradual_reduction(i));
            }
        }
        TacticalResources::new(resources_hash_map)
    }
}
