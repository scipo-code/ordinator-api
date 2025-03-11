pub mod commands;

use std::{fs::File, io::Write};

use anyhow::{bail, Context, Result};
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use commands::Commands;
use reqwest::blocking::Client;
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
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
    }

    let client = create_client()?;

    let system_message = commands::handle_command(cli, &client);

    let response =
        send_http(&client, system_message).context("Imperium did not complete the Request");

    if let Err(error) = response {
        let error = format!("{:?}", error)
            .replace("\\n", "\n")
            .replace("\"", "");
        eprintln!("{}", error);
        std::process::exit(1);
    } else {
        println!("{}", response.unwrap());
    }

    Ok(())
}

fn create_client() -> anyhow::Result<Client> {
    let client = match reqwest::blocking::Client::builder().timeout(None).build() {
        Ok(client) => client,
        Err(e) => bail!("Failed to create HTTP client: {}", e),
    };

    Ok(client)
}

fn send_http(client: &Client, system_message: SystemMessages) -> Result<String> {
    let imperium_address = &dotenvy::var("IMPERIUM_ADDRESS")
        .context("The environment variable IMPERIUM_ADDRESS is not set")?;

    let ordinator_endpoint = &dotenvy::var("ORDINATOR_MAIN_ENDPOINT")
        .context("The environment variable ORDINATOR_MAIN_ENDPOINT is not set")?;

    let base_url = url::Url::parse(&format!("http://{}", imperium_address)).with_context(|| {
        format!(
            "Failed to parse 'IMPERIUM_ADDRESS' as a valid url. Ensure it includes the correct format (e.g. foo:1234). IMPERIUM_ADDRESS: {}", imperium_address
        )
    })?;

    let url = base_url.join(ordinator_endpoint).with_context(|| format!("Failed to append 'ORDINATOR_MAIN_ENDPOINT' as a valid url. Ensure it is correctly formatted. ORDINATOR_MAIN_ENDPOINT: {}", ordinator_endpoint))?;

    let system_message_json_option = serde_json::to_string(&system_message);
    let system_message_json = match system_message_json_option {
        Ok(string) => string,
        Err(_) => {
            bail!("Could not serialize the input response");
        }
    };

    let response = client
        .post(url)
        .body(system_message_json)
        .header("Content-Type", "application/json")
        .send()
        .context("Could not send request")?;

    if !response.status().is_success() {
        bail!(
            "{}, {}",
            response.status(),
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
