mod handlers;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use axum::Router;
use axum::routing::get;
use ordinator_orchestrator::Asset;
// use std::fs::File;
// use std::io::Read;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::TotalSystemSolution;
use routes::api::v1::api_scope;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()>
{
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    // Should the
    // ISSUE #000 Turn the nested `std::sync::Mutex` into `tokio::sync::Mutex`
    let (orchestrator, error_handle): (
        Arc<Orchestrator<TotalSystemSolution>>,
        JoinHandle<Result<()>>,
    ) = Orchestrator::new().context("Orchestrator could not be created")?;

    // WARN: Manually add `Asset`s here. Everything added here should be done from
    // the API in actual production. So this is only a temporary solution.

    orchestrator.asset_factory(&Asset::DF)?;

    // WARN

    
    // HttpServer::new(move || {
    //     App::new()
    //         .app_data(web::Data::new(orchestrator.clone()))
    //         .service(api_scope())
    //         .service(
    //             actix_files::Files::new("/scheduler")
    //                 .index_file("index.html")
    //                 .show_files_listing()
    //                 .use_last_modified(true),
    //         )
    //         .service(
    //             actix_files::Files::new(
    //                 "/supervisor",
    //                 "./static_files/supervisor/dist/supervisor-calendar/browser",
    //             )
    //             .index_file("index.html")
    //             .show_files_listing()
    //             .use_last_modified(true),
    //         )
    //     // TEMP
    //     // `http_to_scheduling_system` is the old entrypoint for the
    //     // `ordinator-imperium` cli tool .route(
    //     //     &dotenvy::var("ORDINATOR_MAIN_ENDPOINT").unwrap(),
    //     //     web::post()
    //     //         .guard(guard::Header("content-type", "application/json"))
    //     //         .to(api::routes::http_to_scheduling_system),
    //     // )
    // })
    // .workers(4)
    // .bind(dotenvy::var("ORDINATOR_API_ADDRESS").unwrap())?
    // .run()
    // .await
    // .map_err(|err| anyhow!(err));
    let scheduler_files = ServeDir::new("./static_files/scheduler/dist");
    let supervisor_files =
        ServeDir::new("./static_files/supervisor/dist/supervisor-calendar/browser");

    let app = Router::new()
        .nest("/api/v1", api_scope(orchestrator.clone()).await)
        .nest_service("/scheduler", scheduler_files)
        .nest_service("/supervisor", supervisor_files)
        .route("/hello", get(|| async { "Hello, world!" }))
        .with_state(orchestrator);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum_server::bind(addr).serve(app.into_make_service());

    tokio::select! {
        res = server => res?,
        res = error_handle => res??,
    }

    Ok(())
}
