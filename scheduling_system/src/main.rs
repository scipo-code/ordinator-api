mod agents;
mod api;
mod data_processing;
mod init;

use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use agents::orchestrator::Orchestrator;

use crate::init::logging;
use agents::orchestrator::ArcOrchestrator;
use shared_types::{scheduling_environment::SchedulingEnvironment, Asset};

///This is the entry point of the application.
#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    dotenvy::dotenv().unwrap();

    let (log_handles, _logging_guard) = logging::setup_logging();

    let database_path_string = &dotenvy::var("DATABASE_PATH").expect("Could not read database path");

    let database_path = std::path::Path::new(database_path_string);

    let scheduling_environment = if database_path.exists() {
        initialize_from_database(database_path)
    } else {
        write_to_database(database_path)
            .expect("Could not write SchedulingEnvironment to database.")
    };

    let mutex_scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

    let mut orchestrator = Orchestrator::new(mutex_scheduling_environment.clone(), log_handles);

    orchestrator.add_asset(Asset::DF);
    orchestrator.initialize_agents_from_env(Asset::DF);

    // orchestrator.add_asset(Asset::HD);

    let arc_orchestrator = ArcOrchestrator(Arc::new(Mutex::new(orchestrator)));

    // let arc_orchestrator_steel = arc_orchestrator.clone();
    // start_steel_repl(arc_orchestrator_steel);

    let arc_orchestrator_server = arc_orchestrator.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(arc_orchestrator_server.0.clone()))
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
}

fn initialize_from_database(path: &Path) -> SchedulingEnvironment {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    serde_json::from_str::<SchedulingEnvironment>(&data).unwrap()
}

fn write_to_database(path: &Path) -> Result<SchedulingEnvironment, std::io::Error> {
    let scheduling_environment =
        init::model_initializers::initialize_scheduling_environment(52, 4, 120);

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
