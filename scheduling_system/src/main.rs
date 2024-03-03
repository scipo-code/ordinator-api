mod agents;
mod api;
mod data_processing;
mod init;
mod models;

use std::sync::{Arc, Mutex};

use crate::init::logging;
use crate::init::ordinator_builder::OrdinatorBuilder;

#[actix_web::main]
async fn main() -> () {
    let _guard = logging::setup_logging();

    let scheduling_environment = Arc::new(Mutex::new(
        init::model_initializers::initialize_scheduling_environment(52),
    ));

    let ordinator_builder = OrdinatorBuilder::new(scheduling_environment).build().await;

    ordinator_builder.await.unwrap();
}

#[cfg(test)]
mod tests {}
