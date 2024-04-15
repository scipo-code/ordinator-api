pub mod commands;

use clap::Parser;
use commands::Commands;
use reqwest::blocking::Client;
use shared_messages::SystemMessages;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Main function of the imperium command line tool
fn main() {
    let cli = Cli::parse();

    let client = reqwest::blocking::Client::new();

    let system_message = commands::handle_command(cli, &client);

    let response = send_http(&client, system_message);

    println!("{}", response);
}

fn send_http(client: &Client, system_message: SystemMessages) -> String {
    let url = "http://localhost:8080/ws";
    let system_message_json = serde_json::to_string(&system_message).unwrap();
    let res = client
        .post(url)
        .body(system_message_json)
        .header("Content-Type", "application/json")
        .send()
        .expect("Could not send request");

    // Check the response status and process the response as needed
    if res.status().is_success() {
        //println!("Request sent successfully");
    } else {
        eprintln!("Failed to send request: {:?}", res.status());
    }
    res.text().unwrap()
}
