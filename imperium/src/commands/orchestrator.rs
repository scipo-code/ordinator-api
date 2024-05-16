use std::collections::HashMap;

use clap::Subcommand;
use clap::{self};
use reqwest::blocking::Client;
use shared_messages::models::time_environment::day::Day;
use shared_messages::models::time_environment::period::Period;
use shared_messages::models::worker_environment::resources;
use shared_messages::{
    models::worker_environment::resources::Id, orchestrator::OrchestratorRequest, SystemMessages,
};
use shared_messages::{Asset, TomlAgents, TomlResources};

#[derive(Subcommand, Debug)]
pub enum OrchestratorCommands {
    /// Status and changes to the scheduling environment
    #[clap(subcommand)]
    SchedulingEnvironment(SchedulingEnvironmentCommands),

    /// Status of the agents
    AgentStatus,

    /// Access the Superviser agent factory
    #[clap(subcommand)]
    SupervisorAgent(SupervisorAgentCommands),
    /// Access the Operational agent factory
    #[clap(subcommand)]
    OperationalAgent(OperationalAgentCommands),

    /// Load a default setup
    InitializeCrewFromFile { asset: Asset, resource_toml: String },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingEnvironmentCommands {
    /// Access the commands to change the work orders (The Orchestrator will ensure that each relevant agent updates its state)
    WorkOrders {
        work_order_number: u32,
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
    ModifyStatusCodes(shared_messages::models::work_order::status_codes::StatusCodes),

    /// Change the unloading point of a work order
    ModifyUnloadingPoint(shared_messages::models::work_order::unloading_point::UnloadingPoint),
}

#[derive(Subcommand, Debug)]
pub enum WorkerEnvironmentCommands {}

#[derive(Subcommand, Debug)]
pub enum TimeEnvironmentCommands {}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {}

#[derive(Subcommand, Debug)]
pub enum SupervisorAgentCommands {
    /// Create a new SupervisorAgent
    Create {
        asset: Asset,
        id_supervisor: String,
        resource: shared_messages::models::worker_environment::resources::MainResources,
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
        resource: Vec<shared_messages::models::worker_environment::resources::Resources>,
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
                        work_order_number,
                        work_order_commands,
                    } => match work_order_commands {
                        WorkOrderCommands::ModifyStatusCodes(status_codes_input) => {
                            let orchestrator_request = OrchestratorRequest::SetWorkOrderState(
                                work_order_number.into(),
                                status_codes_input,
                            );
                            SystemMessages::Orchestrator(orchestrator_request)
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
                    SchedulingEnvironmentCommands::TimeEnvironment(_time_environment_commands) => {
                        todo!()
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
                        id_supervisor: id,
                        resource,
                    } => {
                        let create_supervisor_agent = OrchestratorRequest::CreateSupervisorAgent(
                            asset.clone(),
                            Id::new(id.clone(), vec![], Some(resource.clone())),
                        );
                        SystemMessages::Orchestrator(create_supervisor_agent)
                    }
                    SupervisorAgentCommands::Delete {
                        asset,
                        id_supervisor: id,
                    } => {
                        let delete_supervisor_agent =
                            OrchestratorRequest::DeleteSupervisorAgent(asset.clone(), id.clone());
                        SystemMessages::Orchestrator(delete_supervisor_agent)
                    }
                }
            }
            OrchestratorCommands::OperationalAgent(operational_agent_command) => {
                match operational_agent_command {
                    OperationalAgentCommands::Create {
                        asset,
                        id_operational: id,
                        resource,
                    } => {
                        let create_operational_agent = OrchestratorRequest::CreateOperationalAgent(
                            asset.clone(),
                            Id::new(id.clone(), resource.clone(), None),
                        );
                        SystemMessages::Orchestrator(create_operational_agent)
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
                resource_toml,
            } => {
                let supervisor_resources = [
                    resources::MainResources::MtnMech,
                    resources::MainResources::MtnElec,
                    resources::MainResources::MtnScaf,
                ];

                for (i, resource) in supervisor_resources.iter().enumerate() {
                    let create_supervisor_agent: OrchestratorRequest =
                        OrchestratorRequest::CreateSupervisorAgent(
                            asset.clone(),
                            Id::new(format!("L111000{}", i), vec![], Some(resource.clone())),
                        );
                    let message = SystemMessages::Orchestrator(create_supervisor_agent);
                    crate::send_http(client, message);
                }

                let contents = std::fs::read_to_string(resource_toml).unwrap();
                let config: TomlAgents = toml::from_str(&contents).unwrap();

                for agent in config.operational {
                    let create_operational_agent: OrchestratorRequest =
                        OrchestratorRequest::CreateOperationalAgent(
                            asset.clone(),
                            Id::new(agent.id, agent.resources.resources, None),
                        );
                    let message = SystemMessages::Orchestrator(create_operational_agent);
                    crate::send_http(client, message);
                }

                SystemMessages::Orchestrator(OrchestratorRequest::AgentStatusRequest)
            }
        }
    }
}

pub fn strategic_periods(client: &Client) -> Vec<Period> {
    let orchestrator_request = OrchestratorRequest::GetPeriods;

    let system_message = SystemMessages::Orchestrator(orchestrator_request);

    let strategic_periods_string = crate::send_http(client, system_message);

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

    let tactical_days_string = crate::send_http(client, system_message);
    let tactical_days: HashMap<String, HashMap<String, Vec<Day>>> =
        serde_json::from_str(&tactical_days_string).unwrap();
    tactical_days
        .get("Orchestrator")
        .unwrap()
        .get("Days")
        .unwrap()
        .to_owned()
}
