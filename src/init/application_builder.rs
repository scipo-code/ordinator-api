use core::panic;
use std::{thread, sync::Arc};

use actix::Addr;
use actix_web::{HttpServer, App};
use tracing::{info, Subscriber, event, Level};
use actix_web::web::Data;

use crate::{agents::scheduler_agent::SchedulerAgent, api::routes::ws_index};

pub struct ApplicationBuilder {
    scheduler_agent_addr: Option<Addr<SchedulerAgent>>,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder {
            scheduler_agent_addr: None,
        }
    }

    pub fn with_scheduler_agent(mut self, addr: Addr<SchedulerAgent>) -> Self {
        self.scheduler_agent_addr = Some(addr);
        self
    }
    
    pub async fn build(self) -> Result<(), std::io::Error>{
        info!("Server running at http://127.0.0.1:8001/");
        HttpServer::new(move || {
            let current_thread_id = thread::current().id();
            
            event!(Level::INFO, ?current_thread_id, "initializing application");
            let mut app = App::new();
            
            if let Some(scheduler_agent_addr) = &self.scheduler_agent_addr {
                app = app.app_data(Data::new(Arc::new(scheduler_agent_addr.clone())))
            }
            
            event!(Level::INFO, "about to register routes");
            app.service(ws_index)
        })
        .bind(("127.0.0.1", 8001))?
        .run()
        .await


    }
}