mod agents;
mod api;
mod data_processing;
mod init;

use futures_util::io::AsyncWriteExt;
use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web, App, HttpServer};
use agents::orchestrator::Orchestrator;
use mongodb::{
    bson::{self, doc},
    options::GridFsBucketOptions,
    Client,
};

use shared_messages::{models::SchedulingEnvironment, Asset};
use tracing::info;

use crate::init::logging;

///This is the entry point of the application. We
#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    dotenv::dotenv().ok();

    let log_handles = logging::setup_logging();

    let mongodb_client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();

    // let scheduling_environment: SchedulingEnvironment = match mongodb_client
    //     .database("ordinator").gridfs_bucket(GridFsBucketOptions)
    //     .open_download_stream(, , )
    //     .collection::("fs")
    //     .find_one(None, None)
    //     .await
    //     .unwrap()
    // {
    //     Some(scheduling_environment) => {
    //         info!("SchedulingEnvironment loaded from mongodb");
    //         // retrieve_scheduling_environment(mongodb_client).await
    //         panic!();
    //     }
    //     None => {
    //         let scheduling_environment =
    //             init::model_initializers::initialize_scheduling_environment(52, 4, 120);
    //         store_scheduling_environment(mongodb_client, &scheduling_environment).await;
    //         scheduling_environment
    //     }
    // };
    let scheduling_environment =
        init::model_initializers::initialize_scheduling_environment(52, 4, 120);

    let grib_fs_bucket = mongodb_client
        .database("ordinator")
        .gridfs_bucket(GridFsBucketOptions::default());

    let _bson = bson::to_vec(&scheduling_environment).unwrap();

    let file_id = bson::oid::ObjectId::new();

    grib_fs_bucket.open_upload_stream_with_id(
        bson::Bson::ObjectId(file_id),
        "scheduling_environment",
        None,
    );

    let mutex_scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

    let mut orchestrator = Orchestrator::new(mutex_scheduling_environment.clone(), log_handles);

    orchestrator.add_asset(Asset::DF);
    // orchestrator.add_asset(Asset::HD);
    let arc_orchestrator = Arc::new(Mutex::new(orchestrator));

    HttpServer::new(move || {
        let orchestrator = arc_orchestrator.clone();
        App::new().app_data(web::Data::new(orchestrator)).route(
            "/ws",
            web::post()
                .guard(guard::Header("content-type", "application/json"))
                .to(api::routes::http_to_scheduling_system),
        )
    })
    .workers(4)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn store_scheduling_environment(
    client: Client,
    scheduling_environment: &SchedulingEnvironment,
) {
    let db = client.database("ordinator");

    let grid_fs_bucket = db.gridfs_bucket(GridFsBucketOptions::default());

    let mut upload_stream: mongodb::GridFsUploadStream =
        grid_fs_bucket.open_upload_stream("data/scheduling_environment.dat", None);

    let bincode_scheduling_environment = bincode::serialize(&scheduling_environment).unwrap();

    dbg!(&bincode_scheduling_environment);
    upload_stream
        .write_all(&bincode_scheduling_environment)
        .await
        .unwrap();
    upload_stream.close().await;

    info!("SchedulingEnvironment created from excel data");
}

// async fn retrieve_scheduling_environment(client: Client) -> SchedulingEnvironment {
//     let db = client.database("ordinator");

//     let grid_fs_bucket = db.gridfs_bucket(GridFsBucketOptions::default());

//     let scheduling_environment_id = db.collection("fs");

//     let buffer = String::new();
//     let download_stream: GridFsDownloadStream = grid_fs_bucket.open_download_stream();

//     download_stream.read_to_string()
// }
