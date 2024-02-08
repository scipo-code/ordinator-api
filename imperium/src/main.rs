use clap::{Args, Parser, Subcommand};
use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::{
    strategic_scheduling_message::StrategicSchedulingMessage, StrategicRequest,
};
use shared_messages::FrontendMessages;
use std::net::TcpStream;
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
    Schedule(WorkOrderSchedule),
    /// Lock a period from any scheduling changes
    PeriodLock { period: String },
    /// Exclude a work order from a period
    Exclude { work_order: u32, period: String },
}

#[derive(Debug, Args)]
struct WorkOrderSchedule {
    work_order: u32,
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
        let scheduler_request = StrategicRequest::Status(strategic_status_message);
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
                            StrategicRequest::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                    None => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::General;

                        let front_end_message = FrontendMessages::Strategic(
                            StrategicRequest::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                },
                StrategicSubcommands::Scheduling { subcommand } => match subcommand {
                    Some(SchedulingSubcommands::Schedule(schedule)) => {
                        let schedule_single_work_order =
                            SingleWorkOrder::new(schedule.work_order, schedule.period.clone());

                        let strategic_scheduling_message: StrategicSchedulingMessage =
                            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

                        let strategic_request =
                            StrategicRequest::Scheduling(strategic_scheduling_message);

                        let front_end_message = FrontendMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                    Some(SchedulingSubcommands::PeriodLock { period }) => {
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

                        let front_end_message = FrontendMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
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
