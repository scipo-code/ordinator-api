use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::Result;

use crate::api::FrontendMessages;
use crate::agents::scheduler_agent::SchedulerAgent;
use std::sync::Arc;
pub struct MessageAgent {
    scheduler_agent_addr: Arc<Addr<SchedulerAgent>>,
}



impl Actor for MessageAgent {
    type Context = ws::WebsocketContext<Self>;
}

/// What should I import to make this work correctly? I want to be able to use the SchedulerAgents
/// Addr to send messages to it. Where do I get that from? I think it comes from the Context of the
/// SchedulerAgent that we create in main.rs. How do I get that Context here?
/// 
/// 
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageAgent {
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

impl MessageAgent {
    pub fn new(scheduler_agent_addr: Arc<Addr<SchedulerAgent>>) -> Self {
        MessageAgent {
            scheduler_agent_addr
        }
    }
}