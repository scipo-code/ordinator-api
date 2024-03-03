pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_messages::tactical::TacticalRequest;
use std::sync::{Arc, Mutex};

use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::SchedulingEnvironment;

use crate::agents::strategic_agent::strategic_message::SetAgentAddrMessage;

#[allow(dead_code)]
pub struct TacticalAgent {
    id: i32,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    tactical_algorithm: TacticalAlgorithm,
    ws_addr: Option<Addr<WebSocketAgent>>,
}

impl TacticalAgent {
    pub fn new(
        id: i32,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        addr: Option<Addr<WebSocketAgent>>,
    ) -> Self {
        TacticalAgent {
            id,
            scheduling_environment,
            tactical_algorithm: TacticalAlgorithm::new(),
            ws_addr: addr,
        }
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("WorkPlannerAgent is alive and julia is running");
    }
}

impl Handler<TacticalRequest> for TacticalAgent {
    type Result = ();

    fn handle(
        &mut self,
        tactical_request: TacticalRequest,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        println!("WorkPlannerAgent received WorkPlannerMessage");
        match tactical_request {
            TacticalRequest::Status => {
                let tactical_status = self.tactical_algorithm.status();

                match self.ws_addr.as_ref() {
                    Some(addr) => {
                        addr.do_send(shared_messages::Response::Success(Some(tactical_status)));
                    }
                    None => {
                        println!("No WebSocketAgent address has been provided yet.");
                    }
                }
            }
            TacticalRequest::Scheduling => {
                todo!()
            }
            TacticalRequest::Resources => {
                todo!()
            }
            TacticalRequest::Days => {
                todo!()
            }
        }
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for TacticalAgent {
    type Result = ();

    fn handle(
        &mut self,
        ws_addr_message: SetAgentAddrMessage<WebSocketAgent>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.ws_addr = Some(ws_addr_message.addr);
    }
}
