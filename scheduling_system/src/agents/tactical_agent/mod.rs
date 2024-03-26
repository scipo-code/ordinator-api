pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_messages::agent_error::AgentError;
use shared_messages::resources::Id;
use shared_messages::tactical::TacticalRequest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{instrument, warn};

use crate::agents::strategic_agent::strategic_message::ScheduleIteration;
use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::agents::SetAddr;
use crate::models::SchedulingEnvironment;

use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;
use super::traits::LargeNeighborHoodSearch;
use super::SendState;

#[allow(dead_code)]
pub struct TacticalAgent {
    id: i32,
    time_horizon: u32,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    tactical_algorithm: TacticalAlgorithm,
    strategic_addr: Addr<StrategicAgent>,
    supervisor_addrs: HashMap<Id, Addr<SupervisorAgent>>,
}

impl TacticalAgent {
    pub fn new(
        id: i32,
        days: u32,
        strategic_addr: Addr<StrategicAgent>,
        tactical_algorithm: TacticalAlgorithm,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        TacticalAgent {
            id,
            time_horizon: days,
            scheduling_environment: scheduling_environment.clone(),
            tactical_algorithm,
            strategic_addr,
            supervisor_addrs: HashMap::new(),
        }
    }

    pub fn time_horizon(&self) -> &u32 {
        &self.time_horizon
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        warn!(
            "TacticalAgent {} has started, sending Its address to the StrategicAgent",
            self.id
        );
        self.strategic_addr
            .do_send(SetAddr::Tactical(ctx.address()));
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<ScheduleIteration> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Context<Self>) {
        dbg!();
        self.tactical_algorithm.schedule();
        dbg!();
        ctx.notify(ScheduleIteration {});
        dbg!();
    }
}

impl Handler<TacticalRequest> for TacticalAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        tactical_request: TacticalRequest,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match tactical_request {
            TacticalRequest::Status(tactical_status_message) => self.tactical_algorithm.status(),
            TacticalRequest::Scheduling(tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequest::Resources(tactical_resources_message) => self
                .tactical_algorithm
                .update_resources_state(tactical_resources_message),
            TacticalRequest::Days(tactical_time_message) => {
                todo!()
            }
        }
    }
}

impl Handler<SendState> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, msg: SendState, _ctx: &mut Context<Self>) {
        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
        match msg {
            SendState::Strategic(strategic_state) => {
                let work_orders = scheduling_environment_guard.work_orders().clone();
                drop(scheduling_environment_guard);
                self.tactical_algorithm
                    .update_state_based_on_strategic(&work_orders, strategic_state);
                dbg!();
            }
            SendState::Tactical => {
                todo!()
            }
            SendState::Supervisor => {
                todo!()
            }
            SendState::Operational => {
                todo!()
            }
        }
    }
}

impl Handler<SetAddr> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAddr, _ctx: &mut Context<Self>) {
        match msg {
            SetAddr::Supervisor(id, addr) => {
                self.supervisor_addrs.insert(id, addr);
            }
            _ => {
                println!("The tactical agent received an Addr<T>, where T is not a valid Actor");
                todo!()
            }
        }
    }
}
