pub mod matheuristic;
pub mod messages;
use actix::prelude::*;


use std::sync::Mutex;

use crate::{api::websocket_agent::WebSocketAgent, models::WorkOrders};


pub struct WorkPlannerAgent {
    id: i32,
    work_orders: WorkOrders,
    addr: Option<Addr<WebSocketAgent>>,
}

impl Actor for WorkPlannerAgent {
    type Context = Context<Self>;


    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("WorkPlannerAgent is alive and julia is running");
    }





}