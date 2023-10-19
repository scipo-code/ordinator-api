use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::{HttpRequest, HttpResponse, Result};

use crate::messages::scheduler_message::{SchedulerMessages, RawInputMessage, InputMessage};
use crate::api::FrontendMessages;

pub struct MessageAgent;

impl Actor for MessageAgent {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageAgent {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received length: {}", text.len());
                println!("Received data: {}", text);
                let msg_type: Result<FrontendMessages, serde_json::Error> = serde_json::from_str(&text);
                match msg_type {
                    Ok(FrontendMessages::Scheduler(scheduler_input)) => {
                        handle_scheduler_messages(scheduler_input);
                        ctx.text(text)
                        // Send message to the scheduler agent struct
                    },
                    Ok(FrontendMessages::WorkPlanner) => {
                        handle_work_planner_messages();
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

fn handle_scheduler_messages(scheduler_input: SchedulerMessages) -> () {
    match scheduler_input {
        SchedulerMessages::Input(input_message) => {
            let input_scheduler_data: InputMessage = input_message.into();
            println!("SchedulerAgent received InputMessage");
        },
        SchedulerMessages::WorkPlanner(_work_planner_message) => {
            println!("SchedulerAgent received WorkPlannerMessage");
        },
        SchedulerMessages::Output(_output_message) => {
            println!("SchedulerAgent received OutputMessage");
        }
    }
}

fn handle_work_planner_messages() -> () {
    println!("WorkPlannerAgent received WorkPlannerMessage");
}