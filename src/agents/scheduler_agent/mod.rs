pub mod scheduler_agent;
pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use std::collections::HashMap;
use actix::prelude::*; 

use crate::agents::scheduler_agent::scheduler_message::{SetAgentAddrMessage, SchedulerMessages, InputMessage};
use crate::models::scheduling_environment::WorkOrders;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;

pub struct SchedulerAgent {
    platform: String,
    manual_resources : HashMap<(String, Period), f64>,
    backlog: WorkOrders,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    periods: Vec<Period>,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

impl SchedulerAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }

    // TODO: Here the other Agents Addr messages will also be handled.
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is alive");
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

impl Handler<SchedulerMessages> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SchedulerMessages, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerMessages::Input(msg) => {
                println!("SchedulerAgentReceived a FrontEnd message");
                let input_message: InputMessage = msg.into();
                println!("{}", input_message);

                println!("{:?}", self.manual_resources);
                self.update_scheduler_state(input_message);
                println!("{:?}", self.manual_resources);
                println!("{}", self);

                // TODO - modify state of scheduler agent
            }
            SchedulerMessages::WorkPlanner(msg) => {
               println!("SchedulerAgentReceived a WorkPlannerMessage message");
            },
            SchedulerMessages::ExecuteIteration => {
                // TODO - execute one optimization iteration of the scheduler agent
                self.execute_iteration(ctx);
            }
        }	
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAgentAddrMessage<WebSocketAgent>, ctx: &mut Self::Context) -> Self::Result {
        self.set_ws_agent_addr(msg.addr);
    }
}

impl SchedulerAgent {
    pub fn execute_iteration(&mut self, ctx: &mut <SchedulerAgent as Actor>::Context) {

        println!("I am running a single iteration");  
        ctx.notify(SchedulerMessages::ExecuteIteration)
    }
}

impl SchedulerAgent {
    pub fn new(
        platform: String, 
        manual_resources: HashMap<(String, Period), f64>, 
        backlog: WorkOrders, 
        scheduled_work_orders: HashMap<i32, OrderPeriod>, 
        periods: Vec<Period>,
        ws_agent_addr: Option<Addr<WebSocketAgent>>) 
            -> Self {
  
        Self {
            platform,
            manual_resources,
            backlog,
            scheduled_work_orders,
            periods,
            ws_agent_addr,
        }
    }
}

impl SchedulerAgent {
    pub fn update_scheduler_state(&mut self, input_message: InputMessage) {
        self.manual_resources = input_message.get_manual_resources();
    }
}
