use std::collections::HashMap;
use crate::agents::work_center_agent::WorkCenterAgent;
use crate::models::work_order::WorkOrder;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

use crate::messages::scheduler_message::SchedulerMessages;

use actix::prelude::*; 

pub struct SchedulerAgent {
    platform: String,
    workcenter_agents: HashMap<String, WorkCenterAgent>,
    backlog: Vec<WorkOrder>,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    // inbox: Receiver<SchedulerMessage>,  // Using an mpsc channel for message passing
    periods: Vec<Period>,
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("SchedulerAgent is alive");
    }
}

impl Handler<SchedulerMessages> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SchedulerMessages, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerMessages::Input(input_message) => {
                println!("SchedulerAgent received InputMessage");
                println!("{}", input_message);
            },
            SchedulerMessages::WorkPlanner(_work_planner_message) => {
                println!("SchedulerAgent received WorkPlannerMessage");
            },
            SchedulerMessages::Output(_output_message) => {
                println!("SchedulerAgent received OutputMessage");
            }
            // SchedulerMessages::Output(output_message) => {
            //     println!("SchedulerAgent received OutputMessage");
            // },
            // SchedulerMessages::WorkPlanner(work_planner_message) => {
            //     println!("SchedulerAgent received WorkPlannerMessage");
            // }
        }
    }

}