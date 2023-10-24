use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::Result;
use std::sync::Arc;
use tracing::{Level, event};

use crate::api::FrontendMessages;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::agents::scheduler_agent::scheduler_message::SetAgentAddrMessage;

pub struct WebSocketAgent {
    scheduler_agent_addr: Arc<Addr<SchedulerAgent>>,
}

impl Actor for WebSocketAgent {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        event!(Level::INFO, "WebSocketAgent is alive");
        let addr = ctx.address();
        self.scheduler_agent_addr.do_send(SetAgentAddrMessage { addr });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketAgent {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let msg_type: Result<FrontendMessages, serde_json::Error> = serde_json::from_str(&text);
                match msg_type {
                    Ok(FrontendMessages::Scheduler(scheduler_input)) => {
                        self.scheduler_agent_addr.do_send(scheduler_input);
                        // handle_scheduler_messages(scheduler_input);
                        ctx.text(text)
                        // Send message to the scheduler agent struct
                    },
                    Ok(FrontendMessages::WorkPlanner) => {
                        println!("WorkPlannerAgent received WorkPlannerMessage");
                        ctx.text(text)
                    },
                    Ok(FrontendMessages::Worker) => {
                        println!("WorkerAgent received WorkerMessage");
                        ctx.text(text)
                    },
                    Ok(FrontendMessages::Activity) => {
                        println!("ActivityAgent received ActivityMessage");
                        ctx.text(text)
                    },
                    Ok(FrontendMessages::WorkCenter) => {
                        println!("WorkCenterAgent received WorkCenterMessage");
                        ctx.text(text)
                    },
                    Ok(FrontendMessages::WorkOrder) => {
                        println!("WorkOrderAgent received WorkOrderMessage");
                        ctx.text(text)
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        return;
                    },
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl WebSocketAgent {
    pub fn new(scheduler_agent_addr: Arc<Addr<SchedulerAgent>>) -> Self {
        WebSocketAgent {
            scheduler_agent_addr
        }
    }
}