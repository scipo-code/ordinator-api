use actix::prelude::*;
use actix_web::Result;
use actix_web_actors::ws;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{event, Level};

use crate::agents::scheduler_agent::scheduler_message::SetAgentAddrMessage;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::agents::scheduler_agent::SchedulingOverviewData;
use crate::api::FrontendMessages;

pub struct WebSocketAgent {
    scheduler_agent_addr: Arc<Addr<SchedulerAgent>>,
}

impl Actor for WebSocketAgent {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        event!(Level::INFO, "WebSocketAgent is alive");
        let addr = ctx.address();
        self.scheduler_agent_addr
            .do_send(SetAgentAddrMessage { addr });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketAgent {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let msg_type: Result<FrontendMessages, serde_json::Error> =
                    serde_json::from_str(&text);
                match msg_type {
                    Ok(FrontendMessages::Scheduler(scheduler_input)) => {
                        event!(Level::INFO, scheduler_front_end_message = %scheduler_input, "SchedulerAgent received SchedulerMessage");
                        self.scheduler_agent_addr.do_send(scheduler_input);
                        let addr = ctx.address();
                        self.scheduler_agent_addr
                            .do_send(SetAgentAddrMessage { addr });
                        ctx.text(text)
                    }
                    Ok(FrontendMessages::WorkPlanner) => {
                        println!("WorkPlannerAgent received WorkPlannerMessage");
                        ctx.text(text)
                    }
                    Ok(FrontendMessages::Worker) => {
                        println!("WorkerAgent received WorkerMessage");
                        ctx.text(text)
                    }
                    Ok(FrontendMessages::Activity) => {
                        println!("ActivityAgent received ActivityMessage");
                        ctx.text(text)
                    }
                    Ok(FrontendMessages::WorkCenter) => {
                        println!("WorkCenterAgent received WorkCenterMessage");
                        ctx.text(text)
                    }
                    Ok(FrontendMessages::WorkOrder) => {
                        println!("WorkOrderAgent received WorkOrderMessage");
                        ctx.text(text)
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
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
            scheduler_agent_addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
        event!(Level::INFO, scheduler_front_end_message.websocket = ?msg.scheduling_overview_data.len(), "Scheduler Table data sent to frontend");
        let serialized_message = serde_json::to_string(&msg).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

#[derive(serde::Serialize)]
pub struct SchedulerFrontendLoadingMessage {
    pub frontend_message_type: String,
    pub manual_resources_loading: HashMap<String, HashMap<String, f64>>,
}

impl Message for SchedulerFrontendLoadingMessage {
    type Result = ();
}

impl Handler<SchedulerFrontendLoadingMessage> for WebSocketAgent {
    type Result = ();

    fn handle(
        &mut self,
        msg: SchedulerFrontendLoadingMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&msg).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrders;
    use crate::agents::scheduler_agent::scheduler_algorithm::PriorityQueues;
    use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;
    use crate::agents::scheduler_agent::SchedulerAgent;
    use crate::models::WorkOrders;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    use crate::models::time_environment::period::Period;

    #[actix_rt::test]
    async fn test_websocket_agent() {
        let start_date_str = "2023-10-23T12:00:00Z";
        let start_date: DateTime<Utc> = start_date_str
            .parse()
            .expect("test of start day string failed");

        let end_date: DateTime<Utc> = start_date + chrono::Duration::days(14);

        let periods = vec![Period::new(1, start_date, end_date)];

        let optimized_work_orders: OptimizedWorkOrders = OptimizedWorkOrders::new(HashMap::new());

        let scheduler_agent_addr = SchedulerAgent::new(
            "test".to_string(),
            SchedulerAgentAlgorithm::new(
                0.0,
                HashMap::new(),
                HashMap::new(),
                WorkOrders::new(),
                PriorityQueues::new(),
                optimized_work_orders,
                periods,
                true,
            ),
            None,
            None,
        )
        .start();

        let _ws_agent = WebSocketAgent::new(Arc::new(scheduler_agent_addr.clone()));

        // let mut ws_agent_addr = ws_agent.start();
    }
}
