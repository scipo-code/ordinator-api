use std::collections::HashMap;
use actix::prelude::*; 

use crate::models::work_order::WorkOrder;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

use std::hash::Hash;

use crate::agents::scheduler_agent::scheduler_message::{SchedulerMessages, InputMessage};
use priority_queue::PriorityQueue;

use crate::agents::scheduler_agent::SchedulerAgent;


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
        backlog: Vec<WorkOrder>, 
        scheduled_work_orders: HashMap<i32, OrderPeriod>, 
        periods: Vec<Period> ) 
            -> Self {
  
        Self {
            platform,
            manual_resources,
            backlog,
            scheduled_work_orders,
            periods,
        }
    }
}

