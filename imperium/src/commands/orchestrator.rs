use clap::Subcommand;
use reqwest::blocking::Client;
use shared_messages::{orchestrator::OrchestratorRequest, resources::Id, SystemMessages};

#[derive(Subcommand, Debug)]
pub enum OrchestratorCommands {
    /// Status of the scheduling environment
    SchedulingEnvironment,

    /// Status of the agents
    AgentStatus,

    /// Access the Superviser agent factory
    #[clap(subcommand)]
    SupervisorAgent(SupervisorAgentCommands),
    /// Access the Operational agent factory
    #[clap(subcommand)]
    OperationalAgent(OperationalAgentCommands),

    /// Load a default setup
    LoadDefaultWorkCrew,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {}

#[derive(Subcommand, Debug)]
pub enum SupervisorAgentCommands {
    /// Create a new SupervisorAgent
    Create {
        id: String,
        resource: shared_messages::resources::Resources,
    },

    /// Delete a SupervisorAgent
    Delete { id: String },
}
#[derive(Subcommand, Debug)]
pub enum OperationalAgentCommands {
    /// Create a new OperationalAgent
    Create {
        id: String,
        resource: Vec<shared_messages::resources::Resources>,
    },

    /// Delete an OperationalAgent
    Delete { id: String },
}

impl OrchestratorCommands {
    pub fn execute(&self, client: &Client) -> SystemMessages {
        match self {
            OrchestratorCommands::SchedulingEnvironment => {
                todo!()
            }
            OrchestratorCommands::AgentStatus => {
                let agent_status = OrchestratorRequest::GetAgentStatus;
                SystemMessages::Orchestrator(agent_status)
            }
            OrchestratorCommands::SupervisorAgent(supervisor_agent_command) => {
                match supervisor_agent_command {
                    SupervisorAgentCommands::Create { id, resource } => {
                        let create_supervisor_agent = OrchestratorRequest::CreateSupervisorAgent(
                            Id::new(id.clone(), vec![resource.clone()]),
                        );
                        SystemMessages::Orchestrator(create_supervisor_agent)
                    }
                    SupervisorAgentCommands::Delete { id } => {
                        let delete_supervisor_agent =
                            OrchestratorRequest::DeleteSupervisorAgent(id.clone());
                        SystemMessages::Orchestrator(delete_supervisor_agent)
                    }
                }
            }
            OrchestratorCommands::OperationalAgent(operational_agent_command) => {
                match operational_agent_command {
                    OperationalAgentCommands::Create { id, resource } => {
                        let create_operational_agent = OrchestratorRequest::CreateOperationalAgent(
                            Id::new(id.clone(), resource.clone()),
                        );
                        SystemMessages::Orchestrator(create_operational_agent)
                    }
                    OperationalAgentCommands::Delete { id } => {
                        let delete_operational_agent =
                            OrchestratorRequest::DeleteOperationalAgent(id.clone());
                        SystemMessages::Orchestrator(delete_operational_agent)
                    }
                }
            }
            OrchestratorCommands::LoadDefaultWorkCrew => {
                let supervisor_resources = [
                    shared_messages::resources::Resources::MtnMech,
                    shared_messages::resources::Resources::MtnElec,
                    shared_messages::resources::Resources::MtnScaf,
                ];

                for (i, resource) in supervisor_resources.iter().enumerate() {
                    let create_supervisor_agent: OrchestratorRequest =
                        OrchestratorRequest::CreateSupervisorAgent(Id::new(
                            format!("L111000{}", i),
                            vec![resource.clone()],
                        ));
                    let message = SystemMessages::Orchestrator(create_supervisor_agent);
                    crate::send_http(client, message);
                }

                let operational_resources = [
                    shared_messages::resources::Resources::MtnMech,
                    shared_messages::resources::Resources::MtnElec,
                    shared_messages::resources::Resources::MtnScaf,
                    shared_messages::resources::Resources::MtnCran,
                ];

                let number_of_each_resource = [4, 2, 3, 2];
                let mut counter = 0;
                for i in 0..4 {
                    for _j in 0..number_of_each_resource[i] {
                        counter += 1;
                        let create_operational_agent: OrchestratorRequest =
                            OrchestratorRequest::CreateOperationalAgent(Id::new(
                                format!("L111001{}", counter),
                                vec![operational_resources[i].clone()],
                            ));
                        let message = SystemMessages::Orchestrator(create_operational_agent);
                        crate::send_http(client, message);
                    }
                }

                SystemMessages::Orchestrator(OrchestratorRequest::GetAgentStatus)
            }
        }
    }
}

pub fn strategic_periods(client: &Client) -> Vec<String> {
    let orchestrator_request = OrchestratorRequest::GetPeriods;

    let system_message = SystemMessages::Orchestrator(orchestrator_request);

    let strategic_periods = crate::send_http(client, system_message);
    strategic_periods
        .to_string()
        .replace('\"', "")
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

pub fn tactical_days(client: &Client) -> Vec<String> {
    let orchestrator_request = OrchestratorRequest::GetDays;

    let system_message = SystemMessages::Orchestrator(orchestrator_request);

    let tactical_days = crate::send_http(client, system_message);

    tactical_days
        .to_string()
        .replace('\"', "")
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
