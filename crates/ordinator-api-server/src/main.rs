// WARNING Central location for all TODOs
// Description:
// #xx #yy #zz
// xx:
//     10: Configuration
//     20: Orchestrator
//     30: SchedulingEnvironment
//     40: Actor
//     50: Algorithm
//     60: Solution/Parameters/Options
// yy:
//     10: Strategic
//     20: Tactical
//     30: Supervisor
//     40: Operational
// zz:
//     issue number
// TODO #30 #00 #01 [ ] Move time environment configuraion into SchedulingEnvironment
//
// TODO #10 #00 #02 [ ] Move work order parameters from `./configuration` to `./temp_scheduling_environmen_database`
// TODO #10 #00 #03 [ ] Move the `./configuration/work_order_parameters.json` here.
//
// TODO #60 #10 #01 [ ] Move the `Options` into [`Algorithm`] or [`Actor`]
// TODO #60 #20 #01 [ ] Move the `Options` into [`Algorithm`] or [`Actor`]
// TODO #60 #30 #01 [ ] Move the `Options` into [`Algorithm`] or [`Actor`]
// TODO #60 #40 #01 [ ] Move the `Options` into [`Algorithm`] or [`Actor`]
mod handlers;
mod routes;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::web;
// use actix_web::guard;
// use actix_web::web;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
// use std::fs::File;
// use std::io::Read;
use ordinator_orchestrator::Orchestrator;
use routes::api::v1::api_scope;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    let orchestrator = Orchestrator::new().await;

    // WARN: Manually add `Asset`s here. Everything added here should be done from the
    // API in actual production. So this is only a temporary solution.
    orchestrator.lock().unwrap().asset_factory(Asset::Df)?;

    // WARN

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(orchestrator.clone()))
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
        // `http_to_scheduling_system` is the old entrypoint for the
        // `ordinator-imperium` cli tool .route(
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
