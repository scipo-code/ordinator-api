use actix::dev::MessageResponse;
use actix::dev::OneshotSender;
use actix::fut::wrap_future;
use actix::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::info;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::strategic_message::SetAgentAddrMessage;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::models::SchedulingEnvironment;
use shared_messages::SystemMessages;

pub struct OrchestratorAgent {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    agent_factory: AgentFactory,
    agent_registry: ActorRegistry,
}

impl Actor for OrchestratorAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WebSocketAgent is alive");
        let ws_addr = ctx.address();

        self.agent_registry
            .strategic_agent_addr
            .do_send(SetAgentAddrMessage {
                addr: ws_addr.clone(),
            });

        self.agent_registry
            .tactical_agent_addr
            .do_send(SetAgentAddrMessage {
                addr: ws_addr.clone(),
            });
    }
}

struct ActorRegistry {
    strategic_agent_addr: Addr<StrategicAgent>,
    tactical_agent_addr: Addr<TacticalAgent>,
    supervisor_agent_addrs: HashMap<String, Addr<SupervisorAgent>>,
    operational_agent_addrs: HashMap<String, Addr<OperationalAgent>>,
}

impl ActorRegistry {
    fn new(
        strategic_agent_addr: Addr<StrategicAgent>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> Self {
        ActorRegistry {
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_agent_addrs: HashMap::new(),
            operational_agent_addrs: HashMap::new(),
        }
    }

    fn add_supervisor_agent(&mut self, name: String, addr: Addr<SupervisorAgent>) {
        self.supervisor_agent_addrs.insert(name, addr);
    }

    fn add_operational_agent(&mut self, name: String, addr: Addr<OperationalAgent>) {
        self.operational_agent_addrs.insert(name, addr);
    }
}

impl Handler<SystemMessages> for OrchestratorAgent {
    type Result = Option<shared_messages::Response>;

    fn handle(&mut self, system_messages: SystemMessages, ctx: &mut Self::Context) -> Self::Result {
        match system_messages {
            SystemMessages::Status(status_input) => {
                self.agent_registry.strategic_agent_addr.send(status_input)
            }
            SystemMessages::Strategic(strategic_request) => {
                info!(scheduler_front_end_message = %strategic_request, "SchedulerAgent received SchedulerMessage");
                let send_future = self
                    .agent_registry
                    .strategic_agent_addr
                    .send(strategic_request);

                ctx.wait(send_future.into_actor(self).then(|res, actor, ctx| {
                    match res {
                        Ok(result) => result,
                        Err(err) => {
                            todo!();
                        }
                    }
                    // actix::fut::ready(())
                }))
            }
            SystemMessages::Tactical(tactical_request) => {
                println!("WorkPlannerAgent received WorkPlannerMessage");
                self.agent_registry
                    .tactical_agent_addr
                    .send(tactical_request)
            }
            SystemMessages::Operational => {
                println!("WorkerAgent received WorkerMessage");
                todo!()
            }
        }
    }
}

impl OrchestratorAgent {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        let agent_factory = agent_factory::AgentFactory::new(scheduling_environment.clone());

        let strategic_agent_addr = agent_factory.build_strategic_agent();

        let tactical_agent_addr = agent_factory.build_tactical_agent();

        OrchestratorAgent {
            scheduling_environment,
            agent_factory,
            agent_registry: ActorRegistry::new(strategic_agent_addr, tactical_agent_addr),
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
// impl Handler<OverviewMessage> for OrchestratorAgent {
//     type Result = ();

//     fn handle(&mut self, msg: OverviewMessage, ctx: &mut Self::Context) -> Self::Result {
//         // Serialize the message
//         info!(scheduler_front_end_message.websocket = ?msg.scheduling_overview_data.len(), "Scheduler Table data sent to frontend");
//         let serialized_message = serde_json::to_string(&msg).unwrap();

//         // Send the serialized message to the frontend
//         ctx.text(serialized_message);
//     }
// }

// impl Handler<LoadingMessage> for OrchestratorAgent {
//     type Result = ();

//     fn handle(&mut self, msg: LoadingMessage, ctx: &mut Self::Context) -> Self::Result {
//         // Serialize the message
//         let serialized_message = serde_json::to_string(&msg).unwrap();
//         // Send the serialized message to the frontend
//         ctx.text(serialized_message);
//     }
// }

// impl Handler<PeriodMessage> for OrchestratorAgent {
//     type Result = ();

//     fn handle(&mut self, msg: PeriodMessage, ctx: &mut Self::Context) -> Self::Result {
//         // Serialize the message
//         let serialized_message = serde_json::to_string(&msg).unwrap();
//         // Send the serialized message to the frontend
//         ctx.text(serialized_message);
//     }
// }

impl Handler<shared_messages::Response> for OrchestratorAgent {
    type Result = ();

    fn handle(
        &mut self,
        response: shared_messages::Response,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        // Serialize the message
        let serialized_message = serde_json::to_string(&response.to_string()).unwrap();
        // Send the serialized message to the frontend
        // ctx.(serialized_message);
    }
}

#[cfg(test)]
mod tests {}
