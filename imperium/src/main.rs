pub mod commands;

use std::{fs::File, io::Write};

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use commands::Commands;
use reqwest::blocking::Client;
use shared_types::SystemMessages;
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
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap();

    dbg!();
    let system_message = commands::handle_command(cli, &client);
    dbg!(serde_json::to_string(&system_message).unwrap());

    let response = send_http(&client, system_message);
    dbg!();
}

fn send_http(client: &Client, system_message: SystemMessages) -> String {
    let url = "http://".to_string()
        + &dotenvy::var("ORDINATOR_API_ADDRESS").unwrap()
        + &dotenvy::var("ORDINATOR_MAIN_ENDPOINT").unwrap();
    let system_message_json_option = serde_json::to_string(&system_message);
    let system_message_json = match system_message_json_option {
        Ok(string) => string,
        Err(_) => {
            error!("Bad deserialization");
            "hello".to_string()
        }
    };

    dbg!();
    let res = client
        .post(url)
        .body(system_message_json)
        .header("Content-Type", "application/json")
        .send()
        .expect("Could not send request");

    dbg!(&res);
    // Check the response status and process the response as needed
    let header = res.headers().clone();
    if res.status().is_success() {
        match header.get("Content-Disposition") {
            Some(download_header) => {
                let content = res.bytes().unwrap().clone();
                let mut output = File::create("ordinator_dump.xlsx").unwrap();
                output.write_all(&content).unwrap();
                String::from("Downloaded File")
            }
            None => res.text().unwrap(),
        }
    } else {
        eprintln!("Failed to send request: {:?}", res.status());
        String::from("Failed to get response")
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
