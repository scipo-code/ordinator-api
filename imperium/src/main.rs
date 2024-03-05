pub mod commands;

use clap::{Parser, Subcommand};
use commands::orchestrator::OrchestratorCommands;
use commands::sap::SapCommands;
use commands::strategic::StrategicCommands;
use commands::tactical::TacticalCommands;
use reqwest::Client;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use shared_messages::SystemMessages;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// None => Commands::get_status(client).await,

#[derive(Subcommand)]
enum Commands {
    #[command(name = "default")]
    Default,
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = reqwest::Client::new();

    let system_message = handle_command(cli, &client).await;

    let message = serde_json::to_string(&system_message);

    let response = send_http(&client, message.unwrap()).await;

    let formatted_response = response
        .to_string()
        .replace("\\n", "\n")
        .replace(['\"', '\\'], "");

    println!("{}", formatted_response);
}

async fn handle_command(cli: Cli, client: &Client) -> SystemMessages {
    match &cli.command {
        Commands::Default => {
            let strategic_status_message: StrategicStatusMessage = StrategicStatusMessage::General;
            SystemMessages::Strategic(StrategicRequest::Status(strategic_status_message))
        }
        Commands::Orchestrator {
            orchestrator_commands,
        } => orchestrator_commands.execute(),

        Commands::Strategic { strategic_commands } => strategic_commands.execute(client).await,

        Commands::Tactical { tactical_commands } => tactical_commands.execute(),

        Commands::Supervisor => {
            todo!()
        }

        Commands::Operational => {
            todo!()
        }
        Commands::Sap { sap_commands } => sap_commands.execute(),
    }
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
