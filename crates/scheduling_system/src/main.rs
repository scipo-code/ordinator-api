mod agents;
mod api;
mod orchestrator;

use anyhow::{anyhow, Result};
use api::orchestrator_api::{scheduler_asset_names, scheduler_excel_export};
use data_processing::sources::TimeInput;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use anyhow::Context;
use orchestrator::Orchestrator;

use crate::orchestrator::logging;
use shared_types::{scheduling_environment::SchedulingEnvironment, Asset};

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()
        .expect("You need to provide an .env file. Look at the .env.example if for guidance");

    let (log_handles, _logging_guard) = logging::setup_logging();

    let database_path_string =
        &dotenvy::var("DATABASE_PATH").expect("Could not read database path");

    let database_path = std::path::Path::new(database_path_string);

    let scheduling_environment = if database_path.exists() {
        initialize_from_database(database_path)
    } else {
        write_to_database(database_path)
            .expect("Could not write SchedulingEnvironment to database.")
    };

    let mutex_scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

    let orchestrator =
        Orchestrator::new_with_arc(mutex_scheduling_environment.clone(), log_handles).await;

    let asset_string = dotenvy::var("ASSET").expect("The ASSET environment variable should be set");

    let asset = Asset::new_from_string(asset_string.as_str())
        .expect("Please set a valid ASSET environment variable");

    // WARN START: USED FOR CONVENIENCE
    let system_agents_configuration_toml = dotenvy::var("RESOURCE_CONFIG_INITIALIZATION").expect("A resources configuration file was not read, this is not technically an error but it will be treated as such.");

    let mut system_agents = File::open(system_agents_configuration_toml)?;
    let mut system_agent_bytes: Vec<u8> = Vec::new();

    system_agents.read_to_end(&mut system_agent_bytes)?;

    orchestrator
        .lock()
        .unwrap()
        .asset_factory(asset.clone(), system_agent_bytes)
        .with_context(|| {
            format!(
                "{}: {} could not be added",
                std::any::type_name::<Asset>(),
                asset
            )
        })
        .expect("Could not add asset");

    // This is much more understandable. You initialize all the agents in theb
    // `SchedulingEnvironment` and then you simply create them. This is the
    // way that it should be done.
    orchestrator
        .lock()
        .unwrap()
        .initialize_operational_agents()
        .map_err(|err| anyhow!(err))?;

    // WARN FINISH: USED FOR CONVENIENCE
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(orchestrator.clone()))
            .service(api_scope())
            // TEMP
            .service(
                actix_files::Files::new("/scheduler", "./static_files/scheduler/dist")
                    .index_file("index.html")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(
                actix_files::Files::new(
                    "/supervisor",
                    "./static_files/supervisor/dist/supervisor-calendar/browser",
                )
                .index_file("index.html")
                .show_files_listing()
                .use_last_modified(true),
            )
            // TEMP
            .route(
                &dotenvy::var("ORDINATOR_MAIN_ENDPOINT").unwrap(),
                web::post()
                    .guard(guard::Header("content-type", "application/json"))
                    .to(api::routes::http_to_scheduling_system),
            )
    })
    .workers(4)
    .bind(dotenvy::var("ORDINATOR_API_ADDRESS").unwrap())?
    .run()
    .await
    .map_err(|err| anyhow!(err))
}

fn api_scope() -> actix_web::Scope {
    web::scope("/api")
        .route(
            "/scheduler/export/{asset}",
            web::get().to(scheduler_excel_export),
        )
        .route("/scheduler/assets", web::get().to(scheduler_asset_names))
}
fn initialize_from_database(path: &Path) -> SchedulingEnvironment {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    serde_json::from_str::<SchedulingEnvironment>(&data).unwrap()
}

fn write_to_database(path: &Path) -> Result<SchedulingEnvironment, std::io::Error> {
    let time_input = TimeInput {
        number_of_strategic_periods: 52,
        number_of_tactical_periods: 4,
        number_of_days: 100,
        number_of_supervisor_periods: 3,
    };

    // FIX [ ]
    //
    let scheduling_environment =
        init::model_initializers::initialize_scheduling_environment(time_input);

    let json_scheduling_environment = serde_json::to_string(&scheduling_environment).unwrap();
    let mut file = File::create(path).unwrap();

    file.write_all(json_scheduling_environment.as_bytes())
        .unwrap();
    Ok(scheduling_environment)
}

// fn start_steel_repl(arc_orchestrator: ArcOrchestrator) {
//     thread::spawn(move || {
// let mut steel_engine = steel::steel_vm::engine::Engine::new();
// steel_engine.register_type::<ArcOrchestrator>("Orchestrator?");
// steel_engine.register_fn("actor_registry", ArcOrchestrator::print_actor_registry);
// steel_engine.register_type::<Asset>("Asset?");
// steel_engine.register_fn("Asset", Asset::new_from_string);

// steel_engine.register_external_value("asset::df", Asset::DF);
// steel_engine
//     .register_external_value("orchestrator", arc_orchestrator)
//     .unwrap();

// steel_repl::run_repl(steel_engine).unwrap();
//     });
// }
