use clap::Parser;
use serde_json;
use shared_messages::{FrontendMessages, SchedulerRequests};
use std::{fs, path::Path};
use tungstenite::{connect, Message};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    get_workers: String,
}

enum CliCommands {
    GetWorkers,
    GetWorkOrder,
}

fn main() {
    let args = Cli::parse();

    let get_workers = clap::Arg::new("get_workers")
        .short('w')
        .long("workers")
        .help("Get the list of workers");
    // URL of the WebSocket server

    let server_url = Url::parse("ws://localhost:8001/ws").expect("Invalid URL");

    // Connect to the WebSocket server
    let (mut socket, _response) =
        connect(server_url).expect("Cannot connect to the scheduling system");

    let path = Path::new("./imperium/json_responses/scheduler_requests/input/frontend_input_scheduler_message/frontend_input_scheduler_message.json");

    let json_message: String = fs::read_to_string(path).expect("Unable to read message json file");

    let scheduler_request: FrontendMessages =
        serde_json::from_str(&json_message).expect("Unable to deserialize scheduler request");

    let scheduler_request_json = serde_json::to_string(&scheduler_request).unwrap();

    // Send a message to the server
    socket
        .send(Message::Text(scheduler_request_json))
        .expect("Failed to send a message");

    let response_1 = socket.read().expect("Failed to read message");

    println!("Received: {}", response_1);
    // Close the connection
    socket.close(None).expect("Failed to close the connection");
}
