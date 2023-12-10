pub mod matheuristic;
pub mod messages;
use actix::prelude::*;

use crate::{api::websocket_agent::WebSocketAgent, models::WorkOrders};

#[allow(dead_code)]
pub struct WorkPlannerAgent {
    id: i32,
    work_orders: WorkOrders,
    addr: Option<Addr<WebSocketAgent>>,
}

impl Actor for WorkPlannerAgent {
    type Context = Context<Self>;


    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("WorkPlannerAgent is alive and julia is running");
    }





}