use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::sync::Mutex;
use std::{sync::Arc, thread};
use tokio::task::JoinHandle;
use tracing::{info, trace};

use crate::api::routes::ws_index;
use crate::models::SchedulingEnvironment;

use super::agent_factory::AgentFactory;

pub struct OrdinatorBuilder {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl OrdinatorBuilder {
    pub fn new(
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        agent_factory: AgentFactory,
    ) -> Self {
        OrdinatorBuilder {
            scheduling_environment,
        }
    }

    pub async fn build(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            info!("Server running at http://127.0.0.1:8001/");
            HttpServer::new(move || {
                let current_thread_id = thread::current().id();
                trace!(?current_thread_id, "initializing Ordinator");
                let mut app = App::new();

                app = app.app_data(Data::new(self.scheduling_environment.clone()));

                trace!("about to register routes");
                app.service(ws_index)
            })
            .bind(("0.0.0.0", 8001))
            .expect("Could not bind to port 8001.")
            .run()
            .await
            .expect("Websocket server could not be started.")
        })
    }
}
