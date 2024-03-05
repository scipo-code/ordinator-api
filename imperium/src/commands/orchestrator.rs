use clap::Subcommand;
use reqwest::Client;
use shared_messages::{orchestrator::OrchestratorRequest, SystemMessages};

use crate::send_http;

#[derive(Subcommand, Debug)]
pub enum OrchestratorCommands {
    /// Get the status of a specific WorkOrder
    WorkOrder {
        work_order: u32,
    },
    Periods,
}

impl OrchestratorCommands {
    pub fn execute(&self) -> SystemMessages {
        match self {
            OrchestratorCommands::WorkOrder { work_order } => {
                let environment_status_message: OrchestratorRequest =
                    OrchestratorRequest::GetWorkOrderStatus(*work_order);
                SystemMessages::Orchestrator(environment_status_message)
            }
            OrchestratorCommands::Periods => {
                let environment_status_message = OrchestratorRequest::GetPeriods;
                SystemMessages::Orchestrator(environment_status_message)
            }
        }
    }
}

pub async fn get_periods(client: &Client) -> Vec<String> {
    let status_request = OrchestratorRequest::GetPeriods;

    let front_end_message = SystemMessages::Orchestrator(status_request);

    let status_request_json = serde_json::to_string(&front_end_message).unwrap();

    let response = send_http(client, status_request_json).await;
    response
        .to_string()
        .replace('\"', "")
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
