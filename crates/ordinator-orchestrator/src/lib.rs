mod actor_factory;
pub mod actor_registry;
pub mod database;
pub mod logging;
pub mod model_initializers;

use actor_factory::create_shared_solution_arc_swap;
use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use arc_swap::ArcSwap;
use ordinator_configuration::SystemConfigurations;
use ordinator_contracts::orchestrator::OrchestratorRequest;
use ordinator_contracts::orchestrator::OrchestratorResponse;
use ordinator_operational_actor::messages::OperationalRequestMessage;
use ordinator_operational_actor::messages::OperationalResponseMessage;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
use ordinator_scheduling_environment::worker_environment::crew::OperationalConfigurationAll;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_strategic_actor::StrategicApi;
use ordinator_strategic_actor::messages::StrategicRequestMessage;
use ordinator_strategic_actor::messages::StrategicResponseMessage;
use ordinator_supervisor_actor::SupervisorApi;
use ordinator_supervisor_actor::messages::SupervisorRequestMessage;
use ordinator_tactical_actor::messages::TacticalRequestMessage;
use ordinator_tactical_actor::messages::TacticalResponseMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use tracing::instrument;

use self::actor_registry::ActorRegistry;
use self::database::DataBaseConnection;
use self::logging::LogHandles;

pub struct Orchestrator<Ss> {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub system_solutions: HashMap<Asset, Arc<ArcSwap<Ss>>>,
    pub agent_registries: HashMap<Asset, ActorRegistry>,
    pub system_configurations: HashMap<Asset, Arc<ArcSwap<SystemConfigurations>>>,
    pub database_connections: DataBaseConnection,
    pub actor_notify: Option<Weak<Mutex<Orchestrator<Ss>>>>,
    pub log_handles: LogHandles,
}

pub struct NotifyOrchestrator<Ss>(Arc<Mutex<Orchestrator<Ss>>>);

impl<Ss> Clone for NotifyOrchestrator<Ss> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// WARNING: This should only take immutable references to self!
impl<Ss> OrchestratorNotifier for NotifyOrchestrator<Ss>
where
    Ss: SystemSolutionTrait + Send + Sync + 'static,
{
    fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()> {
        let locked_orchestrator = self.0.lock().unwrap();

        let agent_registry = locked_orchestrator
            .agent_registries
            .get(asset)
            .context("Asset should always be there")?;

        let state_link = ActorMessage::State(StateLink::WorkOrders(ActorSpecific::Strategic(
            work_orders.clone(),
        )));

        agent_registry
            .strategic_agent_sender
            .sender
            .send(state_link)?;

        let state_link = ActorMessage::State(StateLink::WorkOrders(ActorSpecific::Strategic(
            work_orders.clone(),
        )));

        agent_registry
            .tactical_agent_sender
            .sender
            .send(state_link)?;

        for comm in agent_registry.supervisor_agent_senders.values() {
            let state_link = ActorMessage::State(StateLink::WorkOrders(ActorSpecific::Strategic(
                work_orders.clone(),
            )));
            comm.sender.send(state_link)?;
        }

        for addr in agent_registry.operational_agent_senders.values() {
            let state_link = ActorMessage::State(StateLink::WorkOrders(ActorSpecific::Strategic(
                work_orders.clone(),
            )));
            addr.sender.send(state_link)?;
        }

        Ok(())
    }
}

impl<Ss> Orchestrator<Ss> {
    #[instrument(level = "info", skip_all)]
    pub async fn handle(
        &mut self,
        orchestrator_request: OrchestratorRequest,
    ) -> Result<OrchestratorResponse> {
        match orchestrator_request {
            OrchestratorRequest::AgentStatusRequest => {
                // for asset in self.agent_registries.keys() {
                //     let strategic_agent_addr = &self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .strategic_agent_sender;

                //     let tactical_agent_addr = &self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .tactical_agent_sender;

                //     // What should we do here? I think that the best approach will be to make the
                //     // code function
                //     strategic_agent_addr.sender.send(ActorMessage::Actor(
                //         StrategicRequestMessage::Status(StrategicStatusMessage::General),
                //     ))?;

                //     tactical_agent_addr.sender.send(ActorMessage::Actor(
                //         TacticalRequestMessage::Status(TacticalStatusMessage::General),
                //     ))?;

                //     for (_id, addr) in self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .supervisor_agent_senders
                //         .iter()
                //     {
                //         addr.sender
                //             .send(ActorMessage::Actor(SupervisorRequestMessage::Status(
                //                 SupervisorStatusMessage::General,
                //             )))?
                //     }

                //     for (_id, addr) in self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .operational_agent_senders
                //         .iter()
                //     {
                //         addr.sender.send(ActorMessage::Actor(
                //             OperationalRequestMessage::Status(OperationalStatusRequest::General),
                //         ))?;
                //     }

                //     let agent_status = self
                //         .agent_registries
                //         .get(asset)
                //         .expect("Asset should always be present")
                //         .recv_all_agents_status()?;

                //     agent_status_by_asset.insert(asset.clone(), agent_status);
                // }
                // let orchestrator_response_status = AgentStatusResponse::new(agent_status_by_asset);
                // let orchestrator_response =
                //     OrchestratorResponse::AgentStatus(orchestrator_response_status);
                Ok(OrchestratorResponse::Success)
            }
            // Do we want to use this? No.. Or actually yes.. We want to use the...
            // We want to use either the SystemConfiguration, or the ActorEnvironment here. I think
            // that is really the crux of the issue here.
            // QUESTION [ ]
            // How to make the code function correctly with the code here? I think that the best
            // thing to do here is put the actor specification in as part of the database. Why are
            // you hesitant? I am hesitant as I do not know the extend of the issue here. The best
            // thing to do is to make the code run with the, this means that the data should be
            // loaded from the `database` and not simply be a configuration. I think that means
            // that the seperate... You could save a lot of code by making the mongodb at the
            // center of all this... No I think that it is better. Remember that the code should
            // work correctly with the database and the with the.
            //
            // So what is the dataflow here? You
            // You should move the code into the SchedulingEnvironment. The TotalSap should handle the initialization
            //
            OrchestratorRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: &WorkOrders = &scheduling_environment_guard.work_orders;

                let work_order = cloned_work_orders
                    .inner
                    .get(&work_order_number)
                    .with_context(|| {
                        format!(
                            "{:?} is not part of the SchedulingEnvironment",
                            work_order_number
                        )
                    })?;

                let asset = &work_order.work_order_info.functional_location.asset;

                let work_order_configuration = self.system_configurations.get(&asset).unwrap();

                let api_solution = match self.system_solutions.get(asset) {
                    Some(arc_swap_shared_solution) => (arc_swap_shared_solution).load(),
                    None => bail!("Asset: {:?} is not initialzed", &asset),
                };

                // let work_order_response = WorkOrderResponse::new(
                //     work_order,
                //     (**api_solution).clone().into(),
                //     work_order_configurations,
                // );
                bail!("Implement this")
            }
            OrchestratorRequest::GetWorkOrdersState(asset) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: &WorkOrders = &scheduling_environment_guard.work_orders;
                // This is not the correct implementation.
                let work_orders: Vec<_> = cloned_work_orders
                    .inner
                    .iter()
                    .filter(|wo| wo.1.work_order_info.functional_location.asset == asset)
                    .collect();

                let loaded_shared_solution = match self.system_solutions.get(&asset) {
                    Some(arc_swap_shared_solution) => arc_swap_shared_solution.load(),
                    None => bail!("Ordinator has not been initialized for asset: {}", &asset),
                };

                // let work_order_configurations = &work_orders.work_order_configurations;
                // let work_order_responses: HashMap<WorkOrderNumber, WorkOrderResponse> =
                // work_orders     .inner
                //     .iter()
                //     .map(|(work_order_number, work_order)| {
                //         let work_order_response = WorkOrderResponse::new(
                //             work_order,
                //             (**loaded_shared_solution).clone().into(),
                //             work_order_configurations,
                //         );
                //         (*work_order_number, work_order_response)
                //     })
                //     .collect();

                bail!("Implement this");
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard
                    .time_environment
                    .strategic_periods
                    .clone();

                let strategic_periods = OrchestratorResponse::Periods(periods);
                Ok(strategic_periods)
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard
                    .time_environment
                    .tactical_days
                    .clone();

                let tactical_days = OrchestratorResponse::Days(days);
                Ok(tactical_days)
            }
            OrchestratorRequest::CreateSupervisorAgent(
                asset,
                number_of_supervisor_periods,
                id_string,
            ) => {
                // FIX
                // Here you should create the system so that an entry in the
                // `SchedulingEnvironment` is created.
                todo!();
                // FIX
                let notify_orchestrator = NotifyOrchestrator(
                    self.actor_notify
                        .as_ref()
                        .expect("Orchestrator is initialized with the Option::Some variant")
                        .upgrade()
                        .expect("This Weak reference should always be able to be upgraded."),
                );

                // The methods should be defined on the `actor_factory`
                // This should be encapsulated. The factory method and the registry should be of the same process.
                // Should this be inside of the `Orchestrator` or the `ActorFactory`? I think that the. So where should
                // this be defined. I think that the best component is the Orchestrator itself.
                // TODO [x] Make trait
                // TODO [ ] Make method on Orchestrator
                // TODO [ ] Integrate `ActorRegistry`
                //
                // FIX [ ] Make a `self.start_supervisor`

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
            // Do we even want this?
            // Yes it is crucial that `OperationalAgent`s can be created on demand. There is no
            // excuse for not having that function.
            // OrchestratorRequest::CreateOperationalAgent(
            //     asset,
            //     id,
            //     hours_per_day,
            //     operational_configuration,
            // ) => {
            //     // This function should update the scheduling environment and then create
            //     // a function should be called on the scheduling environment to process the
            //     // requests to create an agent.
            //     // FIX
            //     // QUESTION
            //     // What should this function do?
            //     // It creates an `OperationalAgent` but that is not enough.
            //     let response_string = format!("Operational agent created with id {}", id);

            //     let operational_configuration_all = OperationalConfigurationAll::new(
            //         id.clone(),
            //         hours_per_day,
            //         operational_configuration,
            //     );

            //     // WARN
            //     // You should create this so that the whole system is optimized
            //     // you should create the configuration. Let `create_operational_agent`
            //     // borrow it. And then insert it into the `SchedulingEnvironment`.
            //     self.create_operational_agent(&operational_configuration_all)?;
            //     // WARN
            //     // Is this API fault tolerant enough? I am not really sure.
            //     self.scheduling_environment
            //         .lock()
            //         .unwrap()
            //         .worker_environment
            //         .agent_environment
            //         .operational
            //         .insert(id, operational_configuration_all);

            //     let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);

            //     Ok(orchestrator_response)
            // }
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
            OrchestratorRequest::Export(_asset) => {
                panic!();
            }
        }
    }

    // QUESTION
    // Is it correct to remove the agents here? I believe yes, the system have the
    // agents that it does. In the scheduling environment. I do not think that
    // we should move too much with this.
    pub fn initialize_operational_agents(&mut self) -> Result<()> {
        let operational_agents = &self
            .scheduling_environment
            .lock()
            .unwrap()
            .worker_environment
            .agent_environment
            .operational
            .clone();

        // WARN
        // You should always initialize the `SchedulingEnvironment` and make sure that
        // is the single source of truth.
        for operational_agent in operational_agents.values() {
            // QUESTION
            // How should this be build? I think that the best approach will be to make
            // what is the best approach to making something like this work?
            // Maybe it is actually okay to do it like this? This issue is that the
            // `SchedulingEnvironment` might not be updated and this will be a bug
            // later on.
            self.create_operational_agent(operational_agent)?;
        }
        Ok(())
    }

    fn create_operational_agent(
        &mut self,
        operational_agent: &OperationalConfigurationAll,
    ) -> Result<()> {
        let notify_orchestrator = NotifyOrchestrator(
            self.actor_notify
                .as_ref()
                .expect("Orchestrator is initialized with the Option::Some variant")
                .upgrade()
                .expect("This Weak reference should always be able to be updated"),
        );

        let shared_solution = self
            .system_solutions
            .get(
                operational_agent
                    .id
                    .2
                    .first()
                    .expect("TODO: we should not simply grap the first element here."),
            )
            .unwrap()
            .clone();

        let operational_agent_addr = OperationalApi::build_operational_agent(
            &operational_agent.id,
            &self.scheduling_environment.lock().unwrap(),
            shared_solution,
            notify_orchestrator,
        )?;

        let asset = operational_agent
            .id
            .2
            .first()
            .expect("There should always be an asset available");

        self.agent_registries
            .get_mut(asset)
            .unwrap()
            .add_operational_agent(operational_agent.id.clone(), operational_agent_addr);
        Ok(())
    }
}

// You need to decouple the messages from the crates. How should
// that be done? You need to create a trait with the correct kinds
// of... God what is the right path forward here? You should make
// tie them together here. I think that it the best approach.
//
// The idea is that you have a single function and then you decide to
// make this function correctly with the right kind of
impl ActorRegistry {
    fn new(
        strategic_agent_addr: Communication<
            ActorMessage<StrategicRequestMessage>,
            StrategicResponseMessage,
        >,
        tactical_agent_addr: Communication<
            ActorMessage<TacticalRequestMessage>,
            TacticalResponseMessage,
        >,
        supervisor_agent_addrs: HashMap<
            Id,
            Communication<ActorMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
        >,
    ) -> Self {
        ActorRegistry {
            strategic_agent_sender: strategic_agent_addr,
            tactical_agent_sender: tactical_agent_addr,
            supervisor_agent_senders: supervisor_agent_addrs,
            operational_agent_senders: HashMap::new(),
        }
    }

    pub fn add_supervisor_agent(
        &mut self,
        id: Id,
        communication: Communication<
            ActorMessage<SupervisorRequestMessage>,
            SupervisorResponseMessage,
        >,
    ) {
        self.supervisor_agent_senders.insert(id, communication);
    }

    pub fn add_operational_agent(
        &mut self,
        id: Id,
        communication: Communication<
            ActorMessage<OperationalRequestMessage>,
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

impl<Ss> Orchestrator<Ss> {
    pub async fn new() -> Arc<Mutex<Self>> {
        let configurations = SystemConfigurations::read_all_configs().unwrap();

        let (log_handles, _logging_guard) = logging::setup_logging();

        let scheduling_environment = DataBaseConnection::scheduling_environment(configurations);

        let database_connections = DataBaseConnection::new();

        // The configurations are already in place, you should strive to make the system
        // as self contained as possible.
        // This simply initializes the WorkerEnvironment, this should be done in the
        // building of the `SchedulingEnvironment` not in here.

        let orchestrator = Orchestrator {
            scheduling_environment,
            system_solutions: HashMap::new(),
            agent_registries: HashMap::new(),
            log_handles,
            actor_notify: None,
            system_configurations: configurations,
            database_connections,
        };

        // This should be removed. This think that the best options is to
        // This should be implemented as. You should not hard code the
        // creation like this. How should the creation come into existence?
        //
        // I think that the best approach to make a mechanism for creating the
        // correct. The orchestrator is holding all the actors.
        // TODO [ ]
        // Develop a initialization process for the actor factory.
        //
        // Rely on the environment variable, and then provide a manual approach.
        orchestrator
            .lock()
            .unwrap()
            .asset_factory(asset.clone(), system_agent_bytes)
            .with_context(|| {
                format!(
                    "{}: {} could not be added",
                    std::any::type_name::<Asset>(),
                    asset
                )
            })
            .expect("Could not add asset");

        // FIX [ ]
        // FIX THIS QUICK. We need to provide this in a centralized way that is
        // connected to the `Throttling` logic of the application.
        let asset_string =
            dotenvy::var("ASSET").expect("The ASSET environment variable should be set");

        let asset = Asset::new_from_string(asset_string.as_str())
            .expect("Please set a valid ASSET environment variable");
        // This is much more understandable. You initialize all the agents in theb
        // `SchedulingEnvironment` and then you simply create them. This is the
        // way that it should be done.
        orchestrator
            .lock()
            .unwrap()
            .initialize_operational_agents()
            .map_err(|err| anyhow!(err))?;
        let arc_orchestrator = Arc::new(Mutex::new(orchestrator));

        arc_orchestrator.lock().unwrap().actor_notify = Some(Arc::downgrade(&arc_orchestrator));
        arc_orchestrator
    }

    // How should this asset function be implemented. The real question is what
    // should be done about the the files versus bitstream. One thing is for
    // sure if the add_asset function should be reused there can be no file
    // handling inside of it. FIX
    // What the fuck is this? Loading in configurations as a `system_agents_bytes`
    // You are a pathetic idiot! You knew better even when you wrote this. This
    // is a horrible way to live your life, God must be ashamed of you!
    pub fn asset_factory(&mut self, asset: Asset) -> Result<()>
    where
        Ss: SystemSolutionTrait,
    {
        // Initialization should not occur in here. Also the configurations should come
        // in from the
        let shared_solutions_arc_swap = create_shared_solution_arc_swap();

        let notify_orchestrator = NotifyOrchestrator(
            self.actor_notify
                .as_ref()
                .unwrap()
                .upgrade()
                .expect("Weak reference part of initialization"),
        );

        // The ID field is completely defined by the defined by the System configurations here.

        self.start
        // FIX [ ] `self.start_strategic_actor()`
        let strategic_agent_addr = StrategicApi::construct_actor(
            id,
            scheduling_environment_guard,
            shared_solution_arc_swap,
            notify_orchestrator,
            system_configurations,
        );

        // Where should their IDs come from? I think that the best approach is to include them
        // from
        let tactical_agent_addr = tactical_factory(
            id,
            Arc::clone(&self.scheduling_environment),
            shared_solution_arc_swap,
            notify_orchestrator,
            system_configurations,
        );

        let supervisors = scheduling_environment_guard
            .worker_environment
            .agent_environment
            .supervisor
            .clone();

        let mut supervisor_communication = HashMap::<
            Id,
            Communication<ActorMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
        >::new();

        // This is a good sign. It means that the system is performing correctly. What
        // should be done about the code in general?
        // Why is the supervisor no used here? This is also not created in the best way.
        for (id, _supervisor_configuration_all) in supervisors {
            let supervisor_addr = supervisor_factory(
                id,
                Arc::clone(&self.scheduling_environment),
                shared_solution_arc_swap,
                notify_orchestrator,
                system_configurations,
            );
            supervisor_communication.insert(id, supervisor_addr);
        }

        let agent_registry = ActorRegistry::new(
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_communication,
        );

        self.system_solutions
            .insert(asset.clone(), shared_solutions_arc_swap);

        self.agent_registries.insert(asset, agent_registry);
        Ok(())
    }
}

// fn start_steel_repl(arc_orchestrator: ArcOrchestrator) {
//     thread::spawn(move || {
// let mut steel_engine = steel::steel_vm::engine::Engine::new();
// steel_engine.register_type::<ArcOrchestrator>("Orchestrator?");
// steel_engine.register_fn("actor_registry",
// ArcOrchestrator::print_actor_registry); steel_engine.register_type::<Asset>("
// Asset?"); steel_engine.register_fn("Asset", Asset::new_from_string);

// steel_engine.register_external_value("asset::df", Asset::DF);
// steel_engine
//     .register_external_value("orchestrator", arc_orchestrator)
//     .unwrap();

// steel_repl::run_repl(steel_engine).unwrap();
//     });
// }
