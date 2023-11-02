use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::Result;
use std::sync::Arc;
use tracing::{Level, event};

use crate::api::FrontendMessages;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::agents::scheduler_agent::scheduler_message::SetAgentAddrMessage;
use crate::agents::scheduler_agent::SchedulingOverviewData;

pub struct WebSocketAgent {
    scheduler_agent_addr: Arc<Addr<SchedulerAgent>>,
}

impl Actor for WebSocketAgent {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        event!(Level::INFO, "WebSocketAgent is alive");
        // panic!("WebSocketAgent is alive");
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
                        let addr = ctx.address();
                        self.scheduler_agent_addr.do_send(SetAgentAddrMessage { addr });
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SchedulerFrontendMessage {
    pub frontend_message_type: String,
    pub scheduling_overview_data: Vec<SchedulingOverviewData>,
}

/// The Scheduler Output should contain all that is needed to make
impl Message for SchedulerFrontendMessage {
    type Result = ();
}

impl Handler<SchedulerFrontendMessage> for WebSocketAgent {
    type Result = ();

    fn handle(&mut self, msg: SchedulerFrontendMessage, ctx: &mut Self::Context) -> Self::Result {
            // Serialize the message
            let serialized_message = serde_json::to_string(&msg).unwrap();
    
            // Send the serialized message to the frontend
            ctx.text(serialized_message);
        
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use super::*;
    use std::collections::HashMap;
    use crate::agents::scheduler_agent::OptimizedWorkOrders;
    use crate::agents::scheduler_agent::SchedulerAgentAlgorithm;
    use crate::agents::scheduler_agent::SchedulerAgent;
    use crate::agents::scheduler_agent::PriorityQueues;
    use crate::agents::scheduler_agent::OptimizedWorkOrder;
    use crate::models::scheduling_environment::WorkOrders;

    use crate::models::period::Period;

    #[actix_rt::test]
    async fn test_websocket_agent() {
        
        
        let start_date_str = "2023-10-23T12:00:00Z";
        let start_date: DateTime<Utc> = start_date_str.parse().expect("test of start day string failed");
        
        let end_date: DateTime<Utc> = start_date + chrono::Duration::days(14);
        
        let periods = vec![Period::new(1, start_date, end_date)];
        
        let optimized_work_orders: OptimizedWorkOrders = OptimizedWorkOrders::new(HashMap::new());

        let scheduler_agent_addr = SchedulerAgent::new(
            "test".to_string(), 
            SchedulerAgentAlgorithm::new(
                HashMap::new(), 
                HashMap::new(), 
                WorkOrders::new(), 
                PriorityQueues::new(), 
                optimized_work_orders,
                periods),
            None
        ).start();


        
        let ws_agent = WebSocketAgent::new(Arc::new(scheduler_agent_addr.clone()));
        // let mut ws_agent_addr = ws_agent.start();
        
        
        
    }
}