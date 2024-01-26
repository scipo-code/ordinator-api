use clap::{Parser, Subcommand, ValueEnum};
use serde_json;
use shared_messages::{FrontendMessages, SchedulerRequests};
use std::{fs, net::TcpStream, path::Path};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get status of the scheduling system
    Status,
    /// Access the strategic agent
    Strategic {
        #[clap(subcommand)]
        subcommand: Option<StrategicSubcommands>,
    },
    /// Access the tactical agent
    Tactical {},
    /// Access the opertional agents
    Operational,
}

#[derive(Subcommand)]
enum StrategicSubcommands {}

fn main() {
    let cli = Cli::parse();

    let mut socket = create_websocket_client();

    match &cli.command {
        Some(Commands::Status) => {
            Commands::get_status(&mut socket);
        }
        Some(Commands::Strategic { subcommand }) => match subcommand {
            Some(subcommand) => {
                todo!()
            }
            None => {
                todo!()
            }
        },
        Some(Commands::Tactical {}) => {
            println!("Tactical");
        }
        Some(Commands::Operational) => {
            println!("Operational");
        }
        None => {}
    }
    socket.close(None).expect("Failed to close the connection");
}

fn create_websocket_client() -> WebSocket<MaybeTlsStream<TcpStream>> {
    let server_url = Url::parse("ws://localhost:8001/ws").expect("Invalid URL");

    // Connect to the WebSocket server
    let (socket, _response) = connect(server_url).expect("Cannot connect to the scheduling system");
    socket
}

impl Commands {
    fn get_status(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
        let scheduler_request = SchedulerRequests::Status;
        let front_end_message = FrontendMessages::Scheduler(scheduler_request);

        let scheduler_request_json = serde_json::to_string(&front_end_message).unwrap();
        socket
            .send(Message::Text(scheduler_request_json))
            .expect("Failed to send a message");

        let response: Message = socket.read().expect("Failed to read message");
        let formatted_response = response.to_string().replace("\\n", "\n").replace('\"', "");

        println!("{}", formatted_response);
    }
}
