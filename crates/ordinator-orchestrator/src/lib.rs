mod actor_factory;
pub mod actor_registry;
pub mod database;
pub mod logging;
pub mod model_initializers;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::Weak;

pub use actor_factory::TotalSystemSolution;
use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use arc_swap::ArcSwap;
use ordinator_configuration::SystemConfigurations;
use ordinator_contracts::orchestrator::OrchestratorResponse;
use ordinator_operational_actor::OperationalApi;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
pub use ordinator_operational_actor::messages::OperationalRequestMessage;
pub use ordinator_operational_actor::messages::OperationalResponseMessage;
pub use ordinator_operational_actor::messages::requests::OperationalStatusRequest;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::StateLink;
pub use ordinator_orchestrator_actor_traits::SystemSolutions;
pub use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
pub use ordinator_scheduling_environment::time_environment::day::Day;
pub use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
pub use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
pub use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_strategic_actor::StrategicApi;
use ordinator_strategic_actor::algorithm::strategic_solution::StrategicSolution;
pub use ordinator_strategic_actor::messages::StrategicRequestMessage;
pub use ordinator_strategic_actor::messages::StrategicResponseMessage;
use ordinator_supervisor_actor::SupervisorApi;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
pub use ordinator_supervisor_actor::messages::SupervisorRequestMessage;
pub use ordinator_supervisor_actor::messages::SupervisorResponseMessage;
pub use ordinator_supervisor_actor::messages::requests::SupervisorStatusMessage;
pub use ordinator_supervisor_actor::messages::responses::SupervisorResponseStatus;
use ordinator_tactical_actor::TacticalApi;
use ordinator_tactical_actor::algorithm::tactical_solution::TacticalSolution;
pub use ordinator_tactical_actor::messages::TacticalRequestMessage;
pub use ordinator_tactical_actor::messages::TacticalResponseMessage;
pub use ordinator_tactical_actor::messages::requests::TacticalStatusMessage;
use ordinator_total_data_processing::excel_dumps::create_excel_dump;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use self::actor_registry::ActorRegistry;
use self::database::DataBaseConnection;
use self::logging::LogHandles;

pub struct Orchestrator<Ss>
{
    pub scheduling_environment: Arc<std::sync::Mutex<SchedulingEnvironment>>,
    pub system_solutions: std::sync::Mutex<HashMap<Asset, Arc<ArcSwap<Ss>>>>,
    pub actor_registries: std::sync::Mutex<HashMap<Asset, ActorRegistry>>,
    pub system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    pub database_connections: DataBaseConnection,
    pub actor_notify: Option<Weak<Orchestrator<Ss>>>,
    pub log_handles: LogHandles,
}

pub struct NotifyOrchestrator<Ss>(Arc<Orchestrator<Ss>>);

impl<Ss> Clone for NotifyOrchestrator<Ss>
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}

// WARNING: This should only take immutable references to self!
impl<Ss> OrchestratorNotifier for NotifyOrchestrator<Ss>
where
    Ss: SystemSolutions + Send + Sync + 'static,
{
    fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()>
// The function should simply be a fire and forget. We should probably, just send a
    // message to the Orchestrator.
    {
        // It is too late to change this at the moment. You have to do something else
        // instead.

        let actor_registries = self.0.actor_registries.lock().unwrap();
        let actor_registry = actor_registries
            .get(asset)
            .context("Asset should always be there")?;

        let state_link = StateLink::WorkOrders(ActorSpecific::Strategic(work_orders.clone()));

        //
        actor_registry
            .strategic_agent_sender
            .from_orchestrator(state_link);

        let state_link = StateLink::WorkOrders(ActorSpecific::Strategic(work_orders.clone()));

        actor_registry
            .tactical_agent_sender
            .from_orchestrator(state_link);

        for comm in actor_registry.supervisor_agent_senders.values() {
            let state_link = StateLink::WorkOrders(ActorSpecific::Strategic(work_orders.clone()));
            comm.from_orchestrator(state_link);
        }

        for comm in actor_registry.operational_agent_senders.values() {
            let state_link = StateLink::WorkOrders(ActorSpecific::Strategic(work_orders.clone()));
            comm.from_orchestrator(state_link);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest
{
    GetWorkOrderStatus(WorkOrderNumber),
    GetWorkOrdersState(Asset),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
    // InitializeSystemAgentsFromFile(Asset, ActorSpecifications),
    CreateSupervisorAgent(Asset, u64, Id),
    DeleteSupervisorAgent(Asset, String),

    // This should be an API handle not simply
    // CreateOperationalAgent(Asset, Id, f64, OperationalConfiguration),
    DeleteOperationalAgent(Asset, String),
    Export(Asset),
}

// These are basically handlers on the `Orchestrator` I think that they
// should go into the. You have learned so much here but you have to
// keep going. Remember to follow your guts here.
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        > + Send
        + Sync,
{
    #[instrument(level = "info", skip_all)]
    pub async fn handle(
        &self,
        orchestrator_request: OrchestratorRequest,
    ) -> Result<OrchestratorResponse>
    {
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

                //     // What should we do here? I think that the best approach will be to make
                // the     // code function
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
                //
                // OperationalRequestMessage::Status(OperationalStatusRequest::General),
                //         ))?;
                //     }

                //     let agent_status = self
                //         .agent_registries
                //         .get(asset)
                //         .expect("Asset should always be present")
                //         .recv_all_agents_status()?;

                //     agent_status_by_asset.insert(asset.clone(), agent_status);
                // }
                // let orchestrator_response_status =
                // AgentStatusResponse::new(agent_status_by_asset);
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
            // You should move the code into the SchedulingEnvironment. The TotalSap should handle
            // the initialization
            OrchestratorRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: &WorkOrders = &scheduling_environment_guard.work_orders;

                let work_order = cloned_work_orders
                    .inner
                    .get(&work_order_number)
                    .with_context(|| {
                        format!("{work_order_number:?} is not part of the SchedulingEnvironment")
                    })?;

                let asset = &work_order.work_order_info.functional_location.asset;

                let _api_solution = match self.system_solutions.lock().unwrap().get(asset) {
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
                let _work_orders: Vec<_> = cloned_work_orders
                    .inner
                    .iter()
                    .filter(|wo| wo.1.work_order_info.functional_location.asset == asset)
                    .collect();

                let _loaded_shared_solution =
                    match self.system_solutions.lock().unwrap().get(&asset) {
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
                _asset,
                _number_of_supervisor_periods,
                _id_string,
            ) => {
                // FIX
                // Here you should create the system so that an entry in the
                // `SchedulingEnvironment` is created.
                // todo!();
                // FIX
                let _notify_orchestrator = NotifyOrchestrator(
                    self.actor_notify
                        .as_ref()
                        .expect("Orchestrator is initialized with the Option::Some variant")
                        .upgrade()
                        .expect("This Weak reference should always be able to be upgraded."),
                );

                // The methods should be defined on the `actor_factory`
                // This should be encapsulated. The factory method and the registry should be of
                // the same process. Should this be inside of the `Orchestrator`
                // or the `ActorFactory`? I think that the. So where should this
                // be defined. I think that the best component is the Orchestrator itself.
                // TODO [x] Make trait
                // TODO [ ] Make method on Orchestrator
                // TODO [ ] Integrate `ActorRegistry`
                //
                // FIX [ ] Make a `self.start_supervisor`

                // let orchestrator_response =
                // OrchestratorResponse::RequestStatus(response_string);

                Ok(OrchestratorResponse::Todo)
            }
            OrchestratorRequest::DeleteSupervisorAgent(asset, id_string) => {
                let id = self
                    .actor_registries
                    .lock()
                    .unwrap()
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string);

                self.actor_registries
                    .lock()
                    .unwrap()
                    .get_mut(&asset)
                    .unwrap()
                    .supervisor_agent_senders
                    .remove(&id);

                let response_string = format!("Supervisor agent deleted with id {id}");
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
                    .actor_registries
                    .lock()
                    .unwrap()
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string.clone());

                self.actor_registries
                    .lock()
                    .unwrap()
                    .get_mut(&asset)
                    .unwrap()
                    .operational_agent_senders
                    .remove(&id);

                let response_string = format!("Operational agent deleted  with id {id_string}");
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
    // TODO [ ]
    // This should be a part of the asset_builder. Yes that is the correct way of
    // going about it.
    // Do not make a complete builder.
    // FIX You should simply delete this message.
}

// You need to decouple the messages from the crates. How should
// that be done? You need to create a trait with the correct kinds
// of... God what is the right path forward here? You should make
// tie them together here. I think that it the best approach.
//
// The idea is that you have a single function and then you decide to
// make this function correctly with the right kind of
impl ActorRegistry
{
    fn new(
        strategic_agent_addr: Communication<StrategicRequestMessage, StrategicResponseMessage>,
        tactical_agent_addr: Communication<TacticalRequestMessage, TacticalResponseMessage>,
        supervisor_agent_addrs: HashMap<
            Id,
            Communication<SupervisorRequestMessage, SupervisorResponseMessage>,
        >,
        operational_actor_communication: HashMap<
            Id,
            Communication<OperationalRequestMessage, OperationalResponseMessage>,
        >,
    ) -> Self
    {
        ActorRegistry {
            strategic_agent_sender: strategic_agent_addr,
            tactical_agent_sender: tactical_agent_addr,
            supervisor_agent_senders: supervisor_agent_addrs,
            operational_agent_senders: operational_actor_communication,
        }
    }

    pub fn add_supervisor_agent(
        &mut self,
        id: Id,
        communication: Communication<SupervisorRequestMessage, SupervisorResponseMessage>,
    )
    {
        self.supervisor_agent_senders.insert(id, communication);
    }

    pub fn add_operational_agent(
        &mut self,
        id: Id,
        communication: Communication<OperationalRequestMessage, OperationalResponseMessage>,
    )
    {
        self.operational_agent_senders.insert(id, communication);
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> Id
    {
        self.supervisor_agent_senders
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        > + Send
        + Sync
        + 'static,
{
    pub fn new() -> Arc<Self>
    {
        let configurations = SystemConfigurations::read_all_configs().unwrap();

        let (log_handles, _logging_guard) = logging::setup_logging();

        let scheduling_environment =
            DataBaseConnection::scheduling_environment(configurations.clone());

        let database_connections = DataBaseConnection::new();

        // The configurations are already in place, you should strive to make the system
        // as self contained as possible.
        // This simply initializes the WorkerEnvironment, this should be done in the
        // building of the `SchedulingEnvironment` not in here.

        let orchestrator: Arc<Orchestrator<Ss>> = Arc::new_cyclic(|weak_self| Orchestrator {
            scheduling_environment,
            system_solutions: std::sync::Mutex::new(HashMap::new()),
            actor_registries: std::sync::Mutex::new(HashMap::new()),
            log_handles,
            actor_notify: Some(weak_self.clone()),
            system_configurations: configurations,
            database_connections,
        });
        orchestrator
    }

    pub fn asset_factory(&self, asset: &Asset) -> Result<&Self>
    {
        let system_solution = Arc::new(ArcSwap::new(Arc::new(Ss::new())));

        self.system_solutions
            .lock()
            .unwrap()
            .insert(asset.clone(), system_solution);
        let dependencies = self.extract_factory_dependencies(asset)?;

        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

        let strategic_id = scheduling_environment_guard
            .worker_environment
            .actor_specification
            .get(asset)
            .unwrap()
            .strategic
            .id
            .clone();

        let strategic_communication = StrategicApi::construct_actor(
            strategic_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            dependencies.3.clone(),
        )
        .with_context(|| format!("Could not construct StartegicActor {strategic_id}"))?;

        // Where should their IDs come from? I think that the best approach is to
        // include them from
        let tactical_id = scheduling_environment_guard
            .worker_environment
            .actor_specification
            .get(asset)
            .unwrap()
            .tactical
            .id
            .clone();
        let tactical_communication = TacticalApi::construct_actor(
            tactical_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            dependencies.3.clone(),
        )
        .with_context(|| format!("{tactical_id} could not be constructed"))?;

        // This is a good sign. It means that the system is performing correctly. What
        // should be done about the code in general?
        // Why is the supervisor no used here? This is also not created in the best way.
        let supervisors = &scheduling_environment_guard
            .worker_environment
            .actor_specification
            .get(asset)
            .unwrap()
            .supervisors;

        let mut supervisor_communications = HashMap::default();
        for supervisor in supervisors {
            let supervisor_communication = SupervisorApi::construct_actor(
                supervisor.id.clone(),
                dependencies.0.clone(),
                dependencies.1.clone(),
                dependencies.2.clone(),
                dependencies.3.clone(),
            )?;

            supervisor_communications.insert(supervisor.id.clone(), supervisor_communication);
        }

        let operationals = &scheduling_environment_guard
            .worker_environment
            .actor_specification
            .get(asset)
            .unwrap()
            .operational;

        let mut operational_communications = HashMap::default();
        for operational in operationals {
            let operational_communication = OperationalApi::construct_actor(
                operational.id.clone(),
                dependencies.0.clone(),
                dependencies.1.clone(),
                dependencies.2.clone(),
                dependencies.3.clone(),
            )?;

            operational_communications.insert(operational.id.clone(), operational_communication);
        }

        let agent_registry = ActorRegistry::new(
            strategic_communication,
            tactical_communication,
            supervisor_communications,
            operational_communications,
        );

        self.actor_registries
            .lock()
            .unwrap()
            .insert(asset.clone(), agent_registry);
        drop(scheduling_environment_guard);
        Ok(self)
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
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions,
{
    pub fn export_xlsx_solution(&self, asset: Asset) -> Result<(Vec<u8>, String)>
    {
        // let system_solution = self
        //     .system_solutions
        //     .get(&asset)
        //     .with_context(|| {
        //         format!("Could not retrieve the shared_solution for asset
        // {asset:#?}")     })?
        //     .load();

        // This is where it gets a little weird. The handlers should only call methods
        // on the orchestrator.
        // This function should lie in the `orchestrator` crate. How in the world did it
        // ever end up in here
        // let strategic_agent_solution =
        // system_solution.strategic().all_scheduled_tasks();
        // let tactical_agent_solution =
        // system_solution.tactical().all_scheduled_tasks();
        let work_orders = {
            let scheduling_environment_lock = self.scheduling_environment.lock().unwrap();
            scheduling_environment_lock.work_orders.clone()
        };

        let xlsx_filename = create_excel_dump(
            asset.clone(),
            work_orders,
            self.system_solutions
                .lock()
                .unwrap()
                .get(&asset)
                .with_context(|| {
                    format!("You should start up a Scheduling System for Asset {asset}")
                })?
                .load(),
        )
        .unwrap();
        let mut buffer = Vec::new();
        let mut file = File::open(&xlsx_filename).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        std::fs::remove_file(xlsx_filename).expect("The XLSX file could not be deleted");
        let filename = format!("ordinator_xlsx_dump_for_{asset}");
        let http_header = format!("attachment; filename={filename}");

        Ok((buffer, http_header))
    }
}
