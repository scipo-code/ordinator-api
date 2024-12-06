use std::collections::HashMap;

use clap::Subcommand;
use clap::{self};
use reqwest::blocking::Client;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;

use shared_types::scheduling_environment::work_order::status_codes::UserStatusCodes;
use shared_types::scheduling_environment::work_order::unloading_point::UnloadingPoint;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::{Resources, Shift};
use shared_types::{
    orchestrator::OrchestratorRequest, scheduling_environment::worker_environment::resources::Id,
    SystemMessages,
};
use shared_types::{Asset, InputSupervisor, SystemAgents};

#[derive(Subcommand, Debug)]
pub enum OrchestratorCommands {
    /// Status and changes to the scheduling environment
    #[clap(subcommand)]
    SchedulingEnvironment(SchedulingEnvironmentCommands),
    /// Status of the agents
    AgentStatus,
    /// Access the Supervisor agent factory
    #[clap(subcommand)]
    SupervisorAgent(SupervisorAgentCommands),
    /// Access the Operational agent factory
    #[clap(subcommand)]
    OperationalAgent(OperationalAgentCommands),
    /// Load a default setup
    InitializeCrewFromFile {
        asset: Asset,
        resource_configuration_file: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingEnvironmentCommands {
    /// Access the commands to change the work orders (The Orchestrator will ensure that each relevant agent updates its state)
    WorkOrders {
        work_order_number: WorkOrderNumber,
        #[clap(subcommand)]
        work_order_commands: WorkOrderCommands,
    },
    /// Access the commands to change the present workers (The Orchestrator will initilize and deinitialize the relevant agents)
    #[clap(subcommand)]
    WorkerEnvironment(WorkerEnvironmentCommands),
    /// Access the commands to change the time environment. Period size, draft periods, time interval for each agent algorithm
    #[clap(subcommand)]
    TimeEnvironment(TimeEnvironmentCommands),
}

#[derive(Subcommand, Debug)]
pub enum WorkOrderCommands {
    /// Change the status codes of a work order
    ModifyStatusCodes(UserStatusCodes),

    /// Change the unloading point of a work order
    ModifyUnloadingPoint(UnloadingPoint),
}

#[derive(Subcommand, Debug)]
pub enum WorkerEnvironmentCommands {}

#[derive(Subcommand, Debug)]
pub enum TimeEnvironmentCommands {
    /// Get all the available periods in the TimeEnvironment
    GetPeriods,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {}

#[derive(Subcommand, Debug)]
pub enum SupervisorAgentCommands {
    /// Create a new SupervisorAgent
    Create {
        asset: Asset,
        shift: Shift,
        supervisor_id: String,
        resource: Option<Resources>,
        number_of_supervisor_periods: u64,
    },

    /// Delete a SupervisorAgent
    Delete { asset: Asset, id_supervisor: String },
}
#[derive(Subcommand, Debug)]
pub enum OperationalAgentCommands {
    /// Create a new OperationalAgent
    Create {
        asset: Asset,
        id_operational: String,
        shift: Shift,
        resource: Vec<Resources>,
    },

    /// Delete an OperationalAgent
    Delete {
        asset: Asset,
        id_operational: String,
    },
}

impl OrchestratorCommands {
    pub fn execute(self, client: &Client) -> SystemMessages {
        match self {
            OrchestratorCommands::SchedulingEnvironment(scheduling_environment_commands) => {
                match scheduling_environment_commands {
                    SchedulingEnvironmentCommands::WorkOrders {
                        work_order_number: _,
                        work_order_commands,
                    } => match work_order_commands {
                        WorkOrderCommands::ModifyStatusCodes(_status_codes_input) => {
                            todo!()
                        }
                        WorkOrderCommands::ModifyUnloadingPoint(_unloading_point) => {
                            todo!()
                        }
                    },
                    SchedulingEnvironmentCommands::WorkerEnvironment(
                        _worker_environment_commands,
                    ) => {
                        todo!()
                    }
                    SchedulingEnvironmentCommands::TimeEnvironment(time_environment_commands) => {
                        match time_environment_commands {
                            TimeEnvironmentCommands::GetPeriods => {
                                dbg!();
                                println!("Debug message");
                                SystemMessages::Orchestrator(OrchestratorRequest::GetPeriods)
                            }
                        }
                    }
                }
            }
            OrchestratorCommands::AgentStatus => {
                let agent_status = OrchestratorRequest::AgentStatusRequest;
                SystemMessages::Orchestrator(agent_status)
            }
            OrchestratorCommands::SupervisorAgent(supervisor_agent_command) => {
                match supervisor_agent_command {
                    SupervisorAgentCommands::Create {
                        asset,
                        shift: _,
                        resource,
                        supervisor_id,
                        number_of_supervisor_periods,
                    } => {
                        let toml_supervisor = InputSupervisor {
                            id: supervisor_id,
                            resource,
                            number_of_supervisor_periods,
                        };
                        let create_supervisor_agent = OrchestratorRequest::CreateSupervisorAgent(
                            asset.clone(),
                            Id::new(toml_supervisor.id.clone(), vec![], Some(toml_supervisor)),
                        );
                        SystemMessages::Orchestrator(create_supervisor_agent)
                    }
                    SupervisorAgentCommands::Delete {
                        asset,
                        id_supervisor,
                    } => {
                        let delete_supervisor_agent = OrchestratorRequest::DeleteSupervisorAgent(
                            asset.clone(),
                            id_supervisor.clone(),
                        );
                        SystemMessages::Orchestrator(delete_supervisor_agent)
                    }
                }
            }
            OrchestratorCommands::OperationalAgent(operational_agent_command) => {
                match operational_agent_command {
                    OperationalAgentCommands::Create {
                        asset: _,
                        id_operational: _id,
                        shift: _,
                        resource: _,
                    } => {
                        todo!();
                        // let create_operational_agent = OrchestratorRequest::CreateOperationalAgent(
                        //     asset.clone(),
                        //     Id::new(id.clone(), resource.clone(), None),
                        //     OperationalConfiguration::new(Availability::default, TimeInterval::new(NaiveTime::), TimeInterval::new(), TimeInterval::new()),
                        // );
                        // SystemMessages::Orchestrator(create_operational_agent)
                    }
                    OperationalAgentCommands::Delete {
                        asset,
                        id_operational: id,
                    } => {
                        let delete_operational_agent =
                            OrchestratorRequest::DeleteOperationalAgent(asset.clone(), id.clone());
                        SystemMessages::Orchestrator(delete_operational_agent)
                    }
                }
            }
            OrchestratorCommands::InitializeCrewFromFile {
                asset,
                resource_configuration_file: resource_toml,
            } => {
                let contents = std::fs::read_to_string(resource_toml).unwrap();
                let config: SystemAgents = toml::from_str(&contents).unwrap();

                for agent in config.operational {
                    let create_operational_agent: OrchestratorRequest =
                        OrchestratorRequest::CreateOperationalAgent(
                            asset.clone(),
                            Id::new(agent.id, agent.resources.resources, None),
                            agent.operational_configuration,
                        );
                    let message = SystemMessages::Orchestrator(create_operational_agent);
                    crate::send_http(client, message).expect(
                        "Could not initialize the crew from configuration. THIS SHOULD BE CHANGED, move it to the WorkerEnvironment in the SchedulingEnvironment",
                    );
                }

                SystemMessages::Orchestrator(OrchestratorRequest::AgentStatusRequest)
            }
        }
    }
}

pub fn strategic_periods(client: &Client) -> Vec<Period> {
    let orchestrator_request = OrchestratorRequest::GetPeriods;

    let system_message = SystemMessages::Orchestrator(orchestrator_request);

    let strategic_periods_string =
        crate::send_http(client, system_message).expect("Could not receive the StrategicResources");

    let strategic_periods: HashMap<String, HashMap<String, Vec<Period>>> =
        serde_json::from_str(&strategic_periods_string).unwrap();
    strategic_periods
        .get("Orchestrator")
        .unwrap()
        .get("Periods")
        .unwrap()
        .to_owned()
}

pub fn tactical_days(client: &Client) -> Vec<Day> {
    let orchestrator_request = OrchestratorRequest::GetDays;

    let system_message = SystemMessages::Orchestrator(orchestrator_request);

    let tactical_days_string =
        crate::send_http(client, system_message).expect("Could not receive the tactical_days");
    let tactical_days: HashMap<String, HashMap<String, Vec<Day>>> =
        serde_json::from_str(&tactical_days_string).unwrap();
    tactical_days
        .get("Orchestrator")
        .unwrap()
        .get("Days")
        .unwrap()
        .to_owned()
}
