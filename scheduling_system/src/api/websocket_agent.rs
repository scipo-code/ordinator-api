use actix::prelude::*;
use actix_web::Result;
use actix_web_actors::ws;
use std::sync::Arc;
use tracing::info;

use crate::agents::scheduler_agent::scheduler_message::LoadingMessage;
use crate::agents::scheduler_agent::scheduler_message::OverviewMessage;
use crate::agents::scheduler_agent::scheduler_message::PeriodMessage;
use crate::agents::scheduler_agent::scheduler_message::SetAgentAddrMessage;
use crate::agents::scheduler_agent::scheduler_message::SuccesMessage;
use crate::agents::scheduler_agent::StrategicAgent;
use shared_messages::SystemMessages;

pub struct WebSocketAgent {
    scheduler_agent_addr: Arc<Addr<StrategicAgent>>,
}

impl Actor for WebSocketAgent {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WebSocketAgent is alive");
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
                let msg_type: Result<SystemMessages, serde_json::Error> =
                    serde_json::from_str(&text);
                match msg_type {
                    Ok(SystemMessages::Status(status_input)) => {
                        self.scheduler_agent_addr.do_send(status_input);
                    }
                    Ok(SystemMessages::Strategic(scheduler_input)) => {
                        info!(scheduler_front_end_message = %scheduler_input, "SchedulerAgent received SchedulerMessage");
                        self.scheduler_agent_addr.do_send(scheduler_input);

                        let addr = ctx.address();
                        self.scheduler_agent_addr
                            .do_send(SetAgentAddrMessage { addr });
                    }
                    Ok(SystemMessages::Tactical) => {
                        println!("WorkPlannerAgent received WorkPlannerMessage");
                    }
                    Ok(SystemMessages::Operational) => {
                        println!("WorkerAgent received WorkerMessage");
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
    pub fn new(scheduler_agent_addr: Arc<Addr<StrategicAgent>>) -> Self {
        WebSocketAgent {
            scheduler_agent_addr,
        }
    }
}

/// The front end should receive the time and period information from the scheduling environment.
/// What is the best approach of making this happen? I am not sure there are significant benefits
/// to understanding how the central data should be structured. At the moment the scheduler agent
/// hold the information about the periods. This is not the best way of structuring the data flow.
///
/// The problem now becomes that when the scheduling_environment changes, does the scheduler agent
/// changes as well? This is note the case so the question then becomes how we should handle this
/// state change instead. Hmm... That is a very tricky question. Because at the moment the scheduler
/// has to be able to handle the state change. I am thinking about turning the SchedulingEnvironment
/// into an agent, but I cannot assess the implications of this. This was not the ideal approach.
/// I would like it that the SchedulingEnvironment is not an Actor. But is accessed concurrently by
/// the Actors themselves. The question is whether it is possible to handle the state change in a
/// good way for the Actors? Does it go against the Actor model? I am not sure. I think it is
impl Handler<OverviewMessage> for WebSocketAgent {
    type Result = ();

    fn handle(&mut self, msg: OverviewMessage, ctx: &mut Self::Context) -> Self::Result {
        // Serialize the message
        info!(scheduler_front_end_message.websocket = ?msg.scheduling_overview_data.len(), "Scheduler Table data sent to frontend");
        let serialized_message = serde_json::to_string(&msg).unwrap();

        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

impl Handler<LoadingMessage> for WebSocketAgent {
    type Result = ();

    fn handle(&mut self, msg: LoadingMessage, ctx: &mut Self::Context) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&msg).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

impl Handler<SuccesMessage> for WebSocketAgent {
    type Result = ();

    fn handle(&mut self, msg: SuccesMessage, ctx: &mut Self::Context) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&msg).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

impl Handler<PeriodMessage> for WebSocketAgent {
    type Result = ();

    fn handle(&mut self, msg: PeriodMessage, ctx: &mut Self::Context) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&msg).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

impl Handler<shared_messages::Response> for WebSocketAgent {
    type Result = ();

    fn handle(
        &mut self,
        response: shared_messages::Response,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&response.to_string()).unwrap();
        // Send the serialized message to the frontend
        ctx.text(serialized_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::scheduler_agent::scheduler_algorithm::AlgorithmResources;
    use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrders;
    use crate::agents::scheduler_agent::scheduler_algorithm::PriorityQueues;
    use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;
    use crate::agents::scheduler_agent::StrategicAgent;
    use crate::models::SchedulingEnvironment;
    use chrono::{DateTime, Utc};
    use shared_messages::strategic::StrategicRequest;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::models::time_environment::period::Period;
    use std::fs;

    #[actix_rt::test]
    async fn test_websocket_agent() {
        let start_date_str = "2023-10-23T12:00:00Z";
        let start_date: DateTime<Utc> = start_date_str
            .parse()
            .expect("test of start day string failed");

        let end_date: DateTime<Utc> = start_date + chrono::Duration::days(14);

        let periods = vec![Period::new(1, start_date, end_date)];

        let optimized_work_orders: OptimizedWorkOrders = OptimizedWorkOrders::new(HashMap::new());

        let scheduler_agent_addr = StrategicAgent::new(
            "test".to_string(),
            Arc::new(Mutex::new(SchedulingEnvironment::default())),
            SchedulerAgentAlgorithm::new(
                0.0,
                AlgorithmResources::default(),
                AlgorithmResources::default(),
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

    #[test]
    fn test_scheduler_input() {
        let json_message =
            fs::read_to_string("tests/unit_testing/frontend_scheduler.json").unwrap();

        let scheduler_input: StrategicRequest = serde_json::from_str(&json_message).unwrap();

        // How can this deserialization be tested? I am not sure. I know that the message is the
        // correct one but that it is not deserialized correctly.
    }
}
