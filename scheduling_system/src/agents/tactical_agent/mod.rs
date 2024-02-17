pub mod matheuristic;
pub mod messages;
use std::sync::{Arc, Mutex};

use actix::prelude::*;

use crate::{api::websocket_agent::WebSocketAgent, models::SchedulingEnvironment};

#[allow(dead_code)]
pub struct TacticalAgent {
    id: i32,
    work_orders: Arc<Mutex<SchedulingEnvironment>>,
    addr: Option<Addr<WebSocketAgent>>,
}

impl TacticalAgent {
    pub fn new(
        id: i32,
        work_orders: Arc<Mutex<SchedulingEnvironment>>,
        addr: Option<Addr<WebSocketAgent>>,
    ) -> Self {
        TacticalAgent {
            id,
            work_orders,
            addr,
        }
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("WorkPlannerAgent is alive and julia is running");
    }
}
