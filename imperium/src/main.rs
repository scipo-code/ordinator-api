pub mod commands;

use clap::{Args, Parser, Subcommand};
use reqwest::Client;
use shared_messages::orchestrator::OrchestratorRequest;
use shared_messages::resources::Resources;
use shared_messages::strategic::strategic_resources_message::StrategicResourcesMessage;
use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::{
    strategic_scheduling_message::StrategicSchedulingMessage, StrategicRequest,
};
use shared_messages::tactical::TacticalRequest;
use shared_messages::SystemMessages;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Access the orchestrator agent (controls the scheduling environment)
    Orchestrator {
        #[clap(subcommand)]
        orchestrator_commands: Option<OrchestratorCommands>,
    },
    /// Access the strategic agent
    Strategic {
        #[clap(subcommand)]
        strategic_commands: Option<StrategicCommands>,
    },
    /// Access the tactical agent
    Tactical {
        #[clap(subcommand)]
        tactical_commands: Option<TacticalCommands>,
    },
    /// Access the supervisor agents
    Supervisor,
    /// Access the opertional agents
    Operational,
    /// Access the SAP integration (Requires user authorization)
    Sap {
        #[clap(subcommand)]
        sap_commands: Option<SapCommands>,
    },
}

#[derive(Debug, Args)]
struct WorkOrderSchedule {
    work_order: u32,
    period: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = reqwest::Client::new();

    let response = handle_command(cli, &client).await;

    dbg!(response.clone());
    let formatted_response = response
        .to_string()
        .replace("\\n", "\n")
        .replace(['\"', '\\'], "");

    println!("{}", formatted_response);
}

async fn handle_command(cli: Cli, client: &Client) -> String {
    match &cli.command {
        Some(Commands::Orchestrator {
            orchestrator_commands: status_commands,
        }) => match status_commands {
            Some(OrchestratorCommands::WorkOrder { work_order }) => {
                let environment_status_message: OrchestratorRequest =
                    OrchestratorRequest::GetWorkOrderStatus(*work_order);
                let front_end_message = SystemMessages::Orchestrator(environment_status_message);
                let status_request_json = serde_json::to_string(&front_end_message).unwrap();

                println!("{}", status_request_json.clone());
                send_http(client, status_request_json).await
            }
            Some(OrchestratorCommands::Periods) => {
                let environment_status_message = OrchestratorRequest::GetPeriods;
                let front_end_message = SystemMessages::Orchestrator(environment_status_message);
                let status_request_json = serde_json::to_string(&front_end_message).unwrap();
                println!("{}", status_request_json);

                send_http(client, status_request_json).await
            }
            None => Commands::get_status(client).await,
        },
        Some(Commands::Strategic {
            strategic_commands: subcommand,
        }) => match subcommand {
            Some(subcommand) => match subcommand {
                StrategicCommands::Status { subcommand } => match subcommand {
                    Some(StatusStrategic::WorkOrders { period }) => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::new_period(period.to_string());

                        let front_end_message = SystemMessages::Strategic(
                            StrategicRequest::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }
                    None => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::General;

                        let front_end_message = SystemMessages::Strategic(
                            StrategicRequest::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }
                },
                StrategicCommands::Scheduling { subcommand } => match subcommand {
                    Some(SchedulingSubcommands::Schedule(schedule)) => {
                        let schedule_single_work_order =
                            SingleWorkOrder::new(schedule.work_order, schedule.period.clone());

                        let strategic_scheduling_message: StrategicSchedulingMessage =
                            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

                        let strategic_request =
                            StrategicRequest::Scheduling(strategic_scheduling_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();
                        println!("{}", scheduler_request_json);

                        send_http(client, scheduler_request_json).await
                    }
                    Some(SchedulingSubcommands::PeriodLock { period: _ }) => {
                        todo!()
                    }
                    Some(SchedulingSubcommands::Exclude { work_order, period }) => {
                        let exclude_single_work_order =
                            SingleWorkOrder::new(*work_order, period.clone());

                        let strategic_scheduling_message: StrategicSchedulingMessage =
                            StrategicSchedulingMessage::ExcludeFromPeriod(
                                exclude_single_work_order,
                            );

                        let strategic_request =
                            StrategicRequest::Scheduling(strategic_scheduling_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }
                    None => {
                        todo!()
                    }
                },
                StrategicCommands::Resources { subcommand } => match subcommand {
                    Some(ResourcesSubcommands::Loading {
                        periods_end,
                        select_resources,
                    }) => {
                        let resources = match select_resources {
                            Some(select_resources) => {
                                let mut resources: Vec<Resources> = vec![];
                                for resource in select_resources {
                                    resources.push(Resources::new_from_string(resource.clone()));
                                }
                                Some(resources)
                            }
                            None => None,
                        };

                        let strategic_resources_message = StrategicResourcesMessage::GetLoadings {
                            periods_end: periods_end.to_string(),
                            select_resources: resources,
                        };

                        let strategic_request =
                            StrategicRequest::Resources(strategic_resources_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }
                    Some(ResourcesSubcommands::Capacity {
                        periods_end,
                        select_resources,
                    }) => {
                        let resources = match select_resources {
                            Some(select_resources) => {
                                let mut resources: Vec<Resources> = vec![];
                                for resource in select_resources {
                                    resources.push(Resources::new_from_string(resource.clone()));
                                }
                                Some(resources)
                            }
                            None => None,
                        };

                        let strategic_resources_message =
                            StrategicResourcesMessage::GetCapacities {
                                periods_end: periods_end.to_string(),
                                select_resources: resources,
                            };

                        let strategic_request =
                            StrategicRequest::Resources(strategic_resources_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }

                    Some(ResourcesSubcommands::SetCapacity {
                        resource: _,
                        period: _,
                        capacity: _,
                    }) => {
                        todo!()
                    }
                    Some(ResourcesSubcommands::SetCapacityPolicy {
                        resource: _,
                        capacity: _,
                    }) => {
                        todo!()
                    }
                    Some(ResourcesSubcommands::SetCapacityPolicyDefault) => {
                        let resources = generate_manual_resources(client);

                        let strategic_resources_message =
                            StrategicResourcesMessage::new_set_resources(resources.await);

                        let strategic_request =
                            StrategicRequest::Resources(strategic_resources_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);
                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        send_http(client, scheduler_request_json).await
                    }
                    None => {
                        todo!()
                    }
                },
            },
            None => {
                todo!()
                // get_objectives(&mut socket);
            }
        },
        Some(Commands::Tactical { tactical_commands }) => match tactical_commands {
            Some(TacticalCommands::Status) => {
                let tactical_status_message = SystemMessages::Tactical(TacticalRequest::Status);
                let tactical_request_json =
                    serde_json::to_string(&tactical_status_message).unwrap();
                println!("Tactical");

                send_http(client, tactical_request_json).await
            }
            Some(TacticalCommands::Objectives) => {
                todo!()
            }
            None => {
                todo!()
            }
        },
        Some(Commands::Supervisor) => {
            todo!()
        }
        Some(Commands::Operational) => {
            todo!()
        }
        Some(Commands::Sap { sap_commands }) => match sap_commands {
            Some(SapCommands::ExtractFromSap) => {
                let url = "https://help.sap.com/docs/SAP_BUSINESSOBJECTS_BUSINESS_INTELLIGENCE_PLATFORM/9029a149a3314dadb8418a2b4ada9bb8/099046a701cb4014b20123ae31320959.html"; // Replace with the actual SAP authorization URL

                if webbrowser::open(url).is_ok() {
                    println!("Opened {} in the default web browser.", url);
                } else {
                    // There was an error opening the URL
                    println!("Failed to open {}.", url);
                }
                "Opening SAP authorization page".to_string()
            }
            Some(SapCommands::PushStrategicToSap) => {
                todo!()
            }
            Some(SapCommands::PushTacticalToSap) => {
                todo!()
            }
            Some(SapCommands::Operational) => {
                todo!()
            }
            None => {
                todo!()
            }
        },
        None => "No command provided".to_string(),
    }
}

async fn get_periods(client: &Client) -> Vec<String> {
    let status_request = OrchestratorRequest::GetPeriods;

    let front_end_message = SystemMessages::Orchestrator(status_request);

    let status_request_json = serde_json::to_string(&front_end_message).unwrap();
    dbg!(status_request_json.clone());

    let response = send_http(client, status_request_json).await;
    dbg!(response.clone());
    response
        .to_string()
        .replace('\"', "")
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

async fn send_http(client: &Client, message: String) -> String {
    let url = "http://localhost:8080/ws";
    let res = client
        .post(url)
        .body(message)
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Could not send request");

    // Check the response status and process the response as needed
    if res.status().is_success() {
        println!("Request sent successfully");
    } else {
        eprintln!("Failed to send request: {:?}", res.status());
    }
    res.text().await.unwrap()
}

impl Commands {
    async fn get_status(client: &Client) -> String {
        let strategic_status_message = StrategicStatusMessage::General;
        let scheduler_request = StrategicRequest::Status(strategic_status_message);
        let front_end_message = SystemMessages::Strategic(scheduler_request);

        let scheduler_request_json = serde_json::to_string(&front_end_message).unwrap();
        println!("{}", scheduler_request_json);

        send_http(client, scheduler_request_json).await
    }
}

async fn generate_manual_resources(
    client: &Client,
) -> HashMap<shared_messages::resources::Resources, HashMap<String, f64>> {
    let periods: Vec<String> = get_periods(client).await;

    let mut resources_hash_map = HashMap::new();
    for resource in shared_messages::resources::Resources::iter() {
        let mut periods_hash_map = HashMap::new();
        for period in periods.clone() {
            periods_hash_map.insert(period, 300.0);
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    dbg!(resources_hash_map.clone());
    resources_hash_map
}
