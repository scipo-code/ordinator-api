use clap::{Args, Parser, Subcommand};
use shared_messages::resources::Resources;
use shared_messages::status::StatusRequest;
use shared_messages::strategic::strategic_resources_message::StrategicResourcesMessage;
use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::{
    strategic_scheduling_message::StrategicSchedulingMessage, StrategicRequest,
};
use shared_messages::SystemMessages;
use std::collections::HashMap;
use std::net::TcpStream;
use strum::IntoEnumIterator;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;
use webbrowser;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get status of the scheduling system
    Status {
        #[clap(subcommand)]
        status_commands: Option<StatusSchedulingEnvironment>,
    },
    /// Access the strategic agent
    Strategic {
        #[clap(subcommand)]
        subcommand: Option<StrategicSubcommands>,
    },
    /// Access the tactical agent
    Tactical {},
    /// Access the opertional agents
    Operational,

    /// Access the SAP integration (Requires user authorization)
    SAP {
        #[clap(subcommand)]
        subcommand: Option<SAPSubcommands>,
    },
}

#[derive(Subcommand, Debug)]
enum StatusSchedulingEnvironment {
    /// Get the status of a specific WorkOrder
    WorkOrder {
        work_order: u32,
    },
    Periods,
}

#[derive(Subcommand, Debug)]
enum StrategicSubcommands {
    /// overview of the strategic agent
    Status {
        #[clap(subcommand)]
        subcommand: Option<StatusStrategic>,
    },
    /// Scheduling commands
    Scheduling {
        #[clap(subcommand)]
        subcommand: Option<SchedulingSubcommands>,
    },
    /// Resources commands
    Resources {
        #[clap(subcommand)]
        subcommand: Option<ResourcesSubcommands>,
    },
}

#[derive(Subcommand, Debug)]
enum ResourcesSubcommands {
    /// Get the loading of the resources
    Loading {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the capacity of the resources
    Capacity {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Set the capacity of a resource
    SetCapacity {
        resource: String,
        period: String,
        capacity: u32,
    },

    /// Set the capacity policy of a resource (used for operation)
    SetCapacityPolicy { resource: String, capacity: u32 },
    /// Set the capacity policy to default (used for testing)
    SetCapacityPolicyDefault,
}

#[derive(Subcommand, Debug)]
enum StatusStrategic {
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

#[derive(Subcommand, Debug)]
enum SAPSubcommands {
    /// Extract scheduling relevant data from SAP (requires user authorization)
    ExtractFromSAP,

    /// Push the 4M+ (strategic) optimized data to SAP (requires user authorization)
    PushStrategicToSAP,

    /// Push the 5W (tactical) optimized data to SAP (requires user authorization)
    PushTacticalToSAP,

    /// Access the 2WF (operational) opmized data (requires user authorization)
    Operational,
}

#[derive(Debug, Args)]
struct WorkOrderSchedule {
    work_order: u32,
    period: String,
}

fn main() {
    let cli = Cli::parse();

    let mut socket = create_websocket_client();

    let message_sent = handle_command(cli, &mut socket);

    match message_sent {
        Some(_) => {
            let response: Message = socket.read().expect("Failed to read message");
            let formatted_response = response.to_string().replace("\\n", "\n").replace('\"', "");

            println!("{}", formatted_response);
        }
        None => {
            println!("No argument provided, use -h or --help")
        }
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
        let strategic_status_message = StrategicStatusMessage::General;
        let scheduler_request = StrategicRequest::Status(strategic_status_message);
        let front_end_message = SystemMessages::Strategic(scheduler_request);

        let scheduler_request_json = serde_json::to_string(&front_end_message).unwrap();
        println!("{}", scheduler_request_json);

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

fn handle_command(cli: Cli, socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Option<String> {
    match &cli.command {
        Some(Commands::Status { status_commands }) => match status_commands {
            Some(StatusSchedulingEnvironment::WorkOrder { work_order }) => {
                let environment_status_message: StatusRequest =
                    StatusRequest::GetWorkOrderStatus(*work_order);
                let front_end_message = SystemMessages::Status(environment_status_message);
                let status_request_json = serde_json::to_string(&front_end_message).unwrap();

                println!("{}", status_request_json.clone());
                socket.send(Message::Text(status_request_json)).unwrap();
            }
            Some(StatusSchedulingEnvironment::Periods) => {
                let environment_status_message = StatusRequest::GetPeriods;
                let front_end_message = SystemMessages::Status(environment_status_message);
                let status_request_json = serde_json::to_string(&front_end_message).unwrap();
                println!("{}", status_request_json);

                socket.send(Message::Text(status_request_json)).unwrap();
            }
            None => Commands::get_status(socket),
        },
        Some(Commands::Strategic { subcommand }) => match subcommand {
            Some(subcommand) => match subcommand {
                StrategicSubcommands::Status { subcommand } => match subcommand {
                    Some(StatusStrategic::WorkOrders { period }) => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::new_period(period.to_string());

                        let front_end_message = SystemMessages::Strategic(
                            StrategicRequest::Status(strategic_status_message),
                        );

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                    None => {
                        let strategic_status_message: StrategicStatusMessage =
                            StrategicStatusMessage::General;

                        let front_end_message = SystemMessages::Strategic(
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

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();
                        println!("{}", scheduler_request_json);
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

                        let front_end_message = SystemMessages::Strategic(strategic_request);

                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }
                    None => {
                        todo!()
                    }
                },
                StrategicSubcommands::Resources { subcommand } => match subcommand {
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

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
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

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
                    }

                    Some(ResourcesSubcommands::SetCapacity {
                        resource,
                        period,
                        capacity,
                    }) => {
                        todo!()
                    }
                    Some(ResourcesSubcommands::SetCapacityPolicy { resource, capacity }) => {
                        todo!()
                    }
                    Some(ResourcesSubcommands::SetCapacityPolicyDefault) => {
                        let resources = generate_manual_resources(socket);

                        let strategic_resources_message =
                            StrategicResourcesMessage::new_set_resources(resources);

                        let strategic_request =
                            StrategicRequest::Resources(strategic_resources_message);

                        let front_end_message = SystemMessages::Strategic(strategic_request);
                        let scheduler_request_json =
                            serde_json::to_string(&front_end_message).unwrap();

                        socket.send(Message::Text(scheduler_request_json)).unwrap();
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
        Some(Commands::Tactical {}) => {
            println!("Tactical");
        }
        Some(Commands::Operational) => {
            println!("Operational");
        }
        Some(Commands::SAP { subcommand }) => match subcommand {
            Some(SAPSubcommands::ExtractFromSAP) => {
                let url = "https://help.sap.com/docs/SAP_BUSINESSOBJECTS_BUSINESS_INTELLIGENCE_PLATFORM/9029a149a3314dadb8418a2b4ada9bb8/099046a701cb4014b20123ae31320959.html"; // Replace with the actual SAP authorization URL

                // Open the URL in the default web browser
                if webbrowser::open(url).is_ok() {
                    // The URL was opened successfully
                    println!("Opened {} in the default web browser.", url);
                } else {
                    // There was an error opening the URL
                    println!("Failed to open {}.", url);
                }
            }
            Some(SAPSubcommands::PushStrategicToSAP) => {
                todo!()
            }
            Some(SAPSubcommands::PushTacticalToSAP) => {
                todo!()
            }
            Some(SAPSubcommands::Operational) => {
                todo!()
            }
            None => {
                todo!()
            }
        },
        None => return None,
    }
    Some("Message sent".to_string())
}

fn generate_manual_resources(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> HashMap<shared_messages::resources::Resources, HashMap<String, f64>> {
    let periods = get_periods(socket);

    let mut resources_hash_map = HashMap::new();
    for resource in shared_messages::resources::Resources::iter() {
        let mut periods_hash_map = HashMap::new();
        for period in periods.clone() {
            periods_hash_map.insert(period, 300.0);
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    resources_hash_map
}

fn get_periods(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Vec<String> {
    let status_request = StatusRequest::GetPeriods;

    let front_end_message = SystemMessages::Status(status_request);

    let status_request_json = serde_json::to_string(&front_end_message).unwrap();

    socket.send(Message::Text(status_request_json)).unwrap();

    let response: Message = socket.read().expect("Failed to read message");

    response
        .to_string()
        .replace('\"', "")
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
