use actix::Addr;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::sync::Mutex;
use std::{sync::Arc, thread};
use tokio::task::JoinHandle;
use tracing::{info, trace};

use crate::agents::orchestrator_agent::OrchestratorAgent;
use crate::models::SchedulingEnvironment;

// pub struct OrdinatorBuilder {
//     orchestrator_agent: Addr<OrchestratorAgent>,
// }

// impl OrdinatorBuilder {
//     pub fn new(orchestrator_agent: Addr<OrchestratorAgent>) -> Self {
//         OrdinatorBuilder { orchestrator_agent }
//     }

//     pub async fn build(self) -> JoinHandle<()> {
//         tokio::spawn(async move {
//             info!("Server running at http://127.0.0.1:8001/");
//             HttpServer::new(move || {
//                 let current_thread_id = thread::current().id();
//                 trace!(?current_thread_id, "initializing Ordinator");
//                 let mut app = App::new();

//                 app = app.app_data(Data::new(self.orchestrator_agent.clone()));
//                 dbg!();
//                 trace!("about to register routes");
//                 dbg!();
//                 app.service(ws_index)
//             })
//             .bind(("0.0.0.0", 8001))
//             .expect("Could not bind to port 8001.")
//             .run()
//             .await
//             .expect("Websocket server could not be started.")
//         })
//     }
// }
