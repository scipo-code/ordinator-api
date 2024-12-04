pub mod commands;

use std::{borrow::Borrow, fs::File, io::Write};

use anyhow::{bail, Context, Result};
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use commands::Commands;
use reqwest::{
    blocking::{Client, Response},
    header::HeaderValue,
};
use shared_types::SystemMessages;

#[derive(Parser)]
#[command(name = "imperium", author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
    #[command(subcommand)]
    command: Commands,
}

/// Main function of the imperium command line tool
fn main() -> Result<()> {
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

    let system_message = commands::handle_command(cli, &client);

    let response =
        send_http(&client, system_message).context("SystemMessage(s) was not handled correctly")?;
    println!("{}", response);
    Ok(())
}

fn send_http(client: &Client, system_message: SystemMessages) -> Result<String> {
    let url = "http://".to_string()
        + &dotenvy::var("IMPERIUM_ADDRESS")
            .expect("The environment variable IMPERIUM_ADDRESS is not set")
        + &dotenvy::var("ORDINATOR_MAIN_ENDPOINT")
            .expect("The environment variable ORDINATOR_MAIN_ENDPOINT is not set");

    let system_message_json_option = serde_json::to_string(&system_message);
    let system_message_json = match system_message_json_option {
        Ok(string) => string,
        Err(_) => {
            bail!("Could not serialize the input response");
        }
    };

    dbg!(&system_message_json);
    let response = client
        .post(url)
        .body(system_message_json)
        .header("Content-Type", "application/json")
        .send()
        .expect("Could not send request");

    if !response.status().is_success() {
        bail!(
            "Error: No success on the imperium request: {:?}",
            response
                .text()
                .context("Could not extract the JSON from the Response")?
        )
    }

    match response
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .context("Could not convert Content-Disposition to &str")?
    {
        "application/json" => Ok(response.text().unwrap()),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            let content = response.bytes().unwrap().clone();
            let mut output = File::create("ordinator_dump.xlsx").unwrap();
            output.write_all(&content).unwrap();
            Ok(String::from(
                "Received an .xlsx dump from the ordinator-api",
            ))
        }
        _ => todo!(),
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
