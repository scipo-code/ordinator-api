mod agents;
mod api;
mod orchestrator;

use actix_web::guard;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;

use std::fs::File;
use std::io::Read;

use shared_types::Asset;

use self::api::orchestrator_api::scheduler_asset_names;
use self::api::orchestrator_api::scheduler_excel_export;
use self::orchestrator::Orchestrator;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    let orchestrator = Orchestrator::new().await;

    let asset_string = dotenvy::var("ASSET").expect("The ASSET environment variable should be set");

    let asset = Asset::new_from_string(asset_string.as_str())
        .expect("Please set a valid ASSET environment variable");

    // This is so ugly, REMEMBER that when you code shit you have to redo it and that costs a lot of time to
    // fix.
    // WARN START: USED FOR CONVENIENCE
    // TODO [ ]
    // You should not read in files here. That is a horrible place to do it.
    //
    // DEBUG
    // THIS IS THE MOST UGLY THING THAT YOU HAVE EVER CREATED. GOD WOULD BE
    // SO ASHAMED OF YOU! YOU HAVE BEEN PUT HERE FOR A REASON AND YOU WASTE
    // IT LIKE THIS! 
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

// Make API scope for everything. Dall needs to understand all this
fn api_scope() -> actix_web::Scope {
    web::scope("/api")
        .route(
            "/scheduler/export/{asset}",
            web::get().to(scheduler_excel_export),
        )
        .route("/scheduler/assets", web::get().to(scheduler_asset_names))
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
