mod handlers;
mod routes;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::guard;
use actix_web::web;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

use std::fs::File;
use std::io::Read;

use handlers::orchestrator_handlers::scheduler_asset_names;
use handlers::orchestrator_handlers::scheduler_excel_export;
use routes::api::v1::api_scope;

use ordinator_orchestrator::Orchestrator;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    let orchestrator = Orchestrator::new().await;

    HttpServer::new(move || {
        App::new()
            // WARN
            // Here the `Orchestrator` should be made available to the system
            .app_data(web::Data::new(orchestrator.clone()))
            // WARN
            .service(api_scope())
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
        // `http_to_scheduling_system` is the old entrypoint for the `ordinator-imperium` cli tool
        // .route(
        //     &dotenvy::var("ORDINATOR_MAIN_ENDPOINT").unwrap(),
        //     web::post()
        //         .guard(guard::Header("content-type", "application/json"))
        //         .to(api::routes::http_to_scheduling_system),
        // )
    })
    .workers(4)
    .bind(dotenvy::var("ORDINATOR_API_ADDRESS").unwrap())?
    .run()
    .await
    .map_err(|err| anyhow!(err))
}
