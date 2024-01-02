use actix::Addr;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::{sync::Arc, thread};
use tracing::{info, trace};

use crate::models::SchedulingEnvironment;
use crate::{agents::scheduler_agent::SchedulerAgent, api::routes::ws_index};

pub struct ApplicationBuilder {
    scheduler_agent_addr: Option<Addr<SchedulerAgent>>,
    scheduling_environment_addr: Option<Addr<SchedulingEnvironment>>,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder {
            scheduler_agent_addr: None,
            scheduling_environment_addr: None,
        }
    }

    pub fn with_scheduler_agent(mut self, addr: Addr<SchedulerAgent>) -> Self {
        dbg!("with_scheduler_agent");
        self.scheduler_agent_addr = Some(addr);
        self
    }

    pub fn with_scheduling_environment(mut self, addr: Addr<SchedulingEnvironment>) -> Self {
        dbg!("with_scheduler_agent");
        self.scheduling_environment_addr = Some(addr);
        self
    }

    pub async fn build(self) -> Result<(), std::io::Error> {
        info!("Server running at http://127.0.0.1:8001/");
        HttpServer::new(move || {
            let current_thread_id = thread::current().id();
            trace!(?current_thread_id, "initializing application");
            let mut app = App::new();

            if let Some(scheduler_agent_addr) = &self.scheduler_agent_addr {
                app = app.app_data(Data::new(Arc::new(scheduler_agent_addr.clone())))
            }

            if let Some(scheduling_environment_addr) = &self.scheduling_environment_addr {
                app = app.app_data(Data::new(Arc::new(scheduling_environment_addr.clone())))
            }

            trace!("about to register routes");
            app.service(ws_index)
        })
        .bind(("0.0.0.0", 8001))?
        .run()
        .await
    }
}
