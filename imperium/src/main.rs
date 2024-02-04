use clap::{Args, FromArgMatches, Parser, Subcommand, ValueEnum};
use serde_json;
use shared_messages::{
    FrontendMessages, StrategicRequests, StrategicSchedulingMessage, StrategicStatusMessage,
};
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

#[derive(Subcommand, Debug)]
enum StrategicSubcommands {
    /// overview of the strategic agent
    Status {
        #[clap(subcommand)]
        subcommand: Option<StatusSubcommands>,
    },
    /// Scheduling commands
    Scheduling {
        #[clap(subcommand)]
        subcommand: Option<SchedulingSubcommands>,
    },
    /// Resources commands
    Resources,
}

#[derive(Subcommand, Debug)]
enum StatusSubcommands {
    /// List all work orders in a given period
    WorkOrders { period: String },
}

#[derive(Subcommand, Debug)]
enum SchedulingSubcommands {
    /// Schedule a specific work order in a given period
    Schedule(ScheduleStruct),
    /// Lock a period from any scheduling changes
    PeriodLock { period: String },
    /// Exclude a work order from a period
    Exclude { work_order: String, period: String },
}

#[derive(Debug, Args)]
struct ScheduleStruct {
    work_order: String,
    period: String,
}

fn main() {
    let cli = Cli::parse();

    let mut socket = create_websocket_client();

    handle_command(cli, &mut socket);
    let response: Message = socket.read().expect("Failed to read message");
    let formatted_response = response.to_string().replace("\\n", "\n").replace('\"', "");

    println!("{}", formatted_response);

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
        let strategic_status_message = StrategicStatusMessage::General;
        let scheduler_request = StrategicRequests::Status(strategic_status_message);
        let front_end_message = FrontendMessages::Strategic(scheduler_request);

        let scheduler_request_json = serde_json::to_string(&front_end_message).unwrap();
        socket
            .send(Message::Text(scheduler_request_json))
            .expect("Failed to send a message");
    }

    // fn get_objective(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    //     // let scheduler_request = SchedulerRequests::Objectives;
    //     let front_end_message = FrontendMessages::Scheduler(scheduler_request);

    //     let scheduler_request_json = serde_json::to_string(&front_end_message).unwrap();
    //     socket
    //         .send(Message::Text(scheduler_request_json))
    //         .expect("Failed to send a message");

    //     let response: Message = socket.read().expect("Failed to read message");
    //     let formatted_response = response.to_string().replace("\\n", "\n").replace('\"', "");

    //     println!("{}", formatted_response);
    // }
}

fn handle_command(cli: Cli, socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    match &cli.command {
        Some(Commands::Status) => {
            Commands::get_status(socket);
        }
        Some(Commands::Strategic { subcommand }) => match subcommand {
            Some(subcommand) => match subcommand {
                StrategicSubcommands::Status { subcommand } => match subcommand {
                    Some(StatusSubcommands::WorkOrders { period }) => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::new_period(period.to_string());
                        let front_end_message = FrontendMessages::Strategic(
                            StrategicRequests::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();
                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                    None => {
                        todo!()
                    }
                },
                StrategicSubcommands::Scheduling { subcommand } => match subcommand {
                    Some(SchedulingSubcommands::Schedule(schedule)) => {
                        todo!()
                    }
                    Some(SchedulingSubcommands::PeriodLock { period }) => {
                        todo!()
                    }
                    Some(SchedulingSubcommands::Exclude { work_order, period }) => {
                        todo!()
                    }
                    None => {
                        todo!()
                    }
                },
                StrategicSubcommands::Resources => {
                    todo!()
                }
            },
            None => {
                todo!()
                // get_objectives(&mut socket);
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
}
