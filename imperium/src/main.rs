pub mod commands;

use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};
use commands::Commands;
use reqwest::blocking::Client;
use shared_messages::SystemMessages;
use tracing::error;

#[derive(Parser)]
#[command(name = "imperium", author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
    #[command(subcommand)]
    command: Commands,
}

/// Main function of the imperium command line tool
fn main() {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
    } else {
        println!("");
    }

    let client = reqwest::blocking::Client::new();

    let system_message = commands::handle_command(cli, &client);

    let response = send_http(&client, system_message);

    println!("{}", response);
}

fn send_http(client: &Client, system_message: SystemMessages) -> String {
    let url = "http://localhost:8080/ws";
    let system_message_json_option = serde_json::to_string(&system_message);
    let system_message_json = match system_message_json_option {
        Ok(string) => string,
        Err(_) => {
            error!("Bad deserialization");
            "hello".to_string()
        }
    };

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

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
