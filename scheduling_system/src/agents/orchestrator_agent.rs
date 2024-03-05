use actix::prelude::*;
use shared_messages::orchestrator::OrchestratorRequest;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::models::SchedulingEnvironment;

pub struct OrchestratorAgent {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    agent_factory: AgentFactory,
    agent_registry: Arc<RwLock<ActorRegistry>>,
}

impl Actor for OrchestratorAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.agent_registry.write().unwrap().orchestrator_agent_addr = Some(ctx.address());
    }
}

pub struct ActorRegistry {
    orchestrator_agent_addr: Option<Addr<OrchestratorAgent>>,
    strategic_agent_addr: Addr<StrategicAgent>,
    tactical_agent_addr: Addr<TacticalAgent>,
    supervisor_agent_addrs: HashMap<String, Addr<SupervisorAgent>>,
    operational_agent_addrs: HashMap<String, Addr<OperationalAgent>>,
}

impl ActorRegistry {
    fn new(
        orchestrator_agent_addr: Option<Addr<OrchestratorAgent>>,
        strategic_agent_addr: Addr<StrategicAgent>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> Self {
        ActorRegistry {
            orchestrator_agent_addr,
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

    pub fn get_orchestrator_agent_addr(&self) -> Addr<OrchestratorAgent> {
        self.orchestrator_agent_addr.clone().unwrap()
    }

    pub fn get_strategic_agent_addr(&self) -> Addr<StrategicAgent> {
        self.strategic_agent_addr.clone()
    }

    pub fn get_tactical_agent_addr(&self) -> Addr<TacticalAgent> {
        self.tactical_agent_addr.clone()
    }

    pub fn get_supervisor_agent_addr(&self, name: String) -> Addr<SupervisorAgent> {
        self.supervisor_agent_addrs.get(&name).unwrap().clone()
    }

    pub fn get_operational_agent_addr(&self, name: String) -> Addr<OperationalAgent> {
        self.operational_agent_addrs.get(&name).unwrap().clone()
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
            agent_registry: Arc::new(RwLock::new(ActorRegistry::new(
                None,
                strategic_agent_addr,
                tactical_agent_addr,
            ))),
        }
    }

    pub fn get_ref_to_actor_registry(&self) -> Arc<RwLock<ActorRegistry>> {
        self.agent_registry.clone()
    }
}

impl Handler<OrchestratorRequest> for OrchestratorAgent {
    type Result = String;
    fn handle(&mut self, msg: OrchestratorRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            OrchestratorRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                if let Some(work_order_status) = cloned_work_orders.inner.get(&work_order_number) {
                    work_order_status.to_string()
                } else {
                    "Work order not found".to_string()
                }
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_periods();

                let periods_string: String = periods
                    .iter()
                    .map(|period| period.get_period_string())
                    .collect::<Vec<String>>()
                    .join(",");
                dbg!(periods_string.clone());

                periods_string
            }
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

#[cfg(test)]
mod tests {}
