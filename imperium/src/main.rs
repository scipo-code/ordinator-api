pub mod commands;

use clap::{Parser, Subcommand};
use commands::orchestrator::OrchestratorCommands;
use commands::sap::SapCommands;
use commands::status::{StatusCommands, WorkOrders};
use commands::strategic::StrategicCommands;
use commands::tactical::TacticalCommands;
use reqwest::Client;
use shared_messages::orchestrator::OrchestratorRequest;
use shared_messages::{LevelOfDetail, LogLevel, SystemMessages};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// None => Commands::get_status(client).await,

#[derive(Subcommand)]
enum Commands {
    Status {
        #[clap(subcommand)]
        status_commands: StatusCommands,
    },
    /// Access the orchestrator agent (controls the scheduling environment)
    Orchestrator {
        #[clap(subcommand)]
        orchestrator_commands: OrchestratorCommands,
    },
    /// Access the strategic agent
    Strategic {
        #[clap(subcommand)]
        strategic_commands: StrategicCommands,
    },
    /// Access the tactical agent
    Tactical {
        #[clap(subcommand)]
        tactical_commands: TacticalCommands,
    },
    /// Access the supervisor agents
    Supervisor,
    /// Access the opertional agents
    Operational,
    /// Access the SAP integration (Requires user authorization)
    Sap {
        #[clap(subcommand)]
        sap_commands: SapCommands,
    },
    Test,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = reqwest::Client::new();

    let system_message = handle_command(cli, &client).await;

    let response = send_http(&client, system_message).await;

    let formatted_response = response.replace('\"', "");

    println!("{}", formatted_response);
}

async fn handle_command(cli: Cli, client: &Client) -> SystemMessages {
    match &cli.command {
        Commands::Status { status_commands } => match status_commands {
            StatusCommands::WorkOrders { work_orders } => match work_orders {
                WorkOrders::WorkOrderState { level_of_detail } => {
                    let orchestrator_request =
                        OrchestratorRequest::GetWorkOrdersState(level_of_detail.clone());
                    SystemMessages::Orchestrator(orchestrator_request)
                }
                WorkOrders::WorkOrder {
                    work_order_number,
                    level_of_detail,
                } => {
                    let strategic_status_message = OrchestratorRequest::GetWorkOrderStatus(
                        *work_order_number,
                        level_of_detail.clone(),
                    );
                    SystemMessages::Orchestrator(strategic_status_message)
                }
            },
            StatusCommands::Workers => {
                todo!()
            }
            StatusCommands::Time {} => {
                todo!()
            }
            StatusCommands::Log { level } => {
                SystemMessages::Orchestrator(OrchestratorRequest::SetLogLevel(level.clone()))
            }
            StatusCommands::Profiling { level } => {
                SystemMessages::Orchestrator(OrchestratorRequest::SetProfiling(level.clone()))
            }
        },
        Commands::Orchestrator {
            orchestrator_commands,
        } => orchestrator_commands.execute(client).await,

        Commands::Strategic { strategic_commands } => strategic_commands.execute(client).await,

        Commands::Tactical { tactical_commands } => tactical_commands.execute(),

        Commands::Supervisor => {
            todo!()
        }
        Commands::Operational => {
            todo!()
        }
        Commands::Sap { sap_commands } => sap_commands.execute(),
        Commands::Test => {
            println!("Hello this is a test");
            todo!();
        }
    }
}

async fn send_http(client: &Client, system_message: SystemMessages) -> String {
    let url = "http://localhost:8080/ws";
    let system_message_json = serde_json::to_string(&system_message).unwrap();
    let res = client
        .post(url)
        .body(system_message_json)
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
