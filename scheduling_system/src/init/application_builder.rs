use actix::Addr;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::{sync::Arc, thread};
use tokio::task::JoinHandle;
use tracing::{info, trace};

use crate::agents::tactical_agent::{self, TacticalAgent};
use crate::{agents::scheduler_agent::StrategicAgent, api::routes::ws_index};

pub struct ApplicationBuilder {
    scheduler_agent_addr: Option<Addr<StrategicAgent>>,
    tactical_agent_addr: Option<Addr<TacticalAgent>>,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder {
            scheduler_agent_addr: None,
            tactical_agent_addr: None,
        }
    }

    pub fn with_scheduler_agent(mut self, addr: Addr<StrategicAgent>) -> Self {
        self.scheduler_agent_addr = Some(addr);
        self
    }

    pub fn with_tactical_agent(self, tactical_agent_addr: Addr<TacticalAgent>) -> Self {
        self.with_tactical_agent(tactical_agent_addr);
        self
    }

    pub async fn build(self) -> JoinHandle<()> {
        let scheduler_agent_addr_clone = self.scheduler_agent_addr.clone();
        let tactical_agent_addr_clone = self.tactical_agent_addr.clone();
        tokio::spawn(async move {
            info!("Server running at http://127.0.0.1:8001/");
            HttpServer::new(move || {
                let current_thread_id = thread::current().id();
                trace!(?current_thread_id, "initializing application");
                let mut app = App::new();

                if let Some(scheduler_agent_addr) = &scheduler_agent_addr_clone {
                    app = app.app_data(Data::new(Arc::new(scheduler_agent_addr.clone())))
                }

                if let Some(tactical_agent_addr) = &tactical_agent_addr_clone {
                    app = app.app_data(Data::new(Arc::new(tactical_agent_addr.clone())))
                }

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
