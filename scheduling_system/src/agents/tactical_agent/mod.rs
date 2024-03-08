pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_messages::resources::Id;
use shared_messages::tactical::TacticalRequest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::agents::SetAddr;
use crate::models::SchedulingEnvironment;

use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;

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
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        TacticalAgent {
            id,
            time_horizon: days,
            scheduling_environment,
            tactical_algorithm: TacticalAlgorithm::new(),
            strategic_addr,
            supervisor_addrs: HashMap::new(),
        }
    }

    pub fn get_time_horizon(&self) -> u32 {
        self.time_horizon
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.strategic_addr
            .do_send(SetAddr::SetTactical(ctx.address()));
    }
}

impl Handler<TacticalRequest> for TacticalAgent {
    type Result = String;

    fn handle(
        &mut self,
        tactical_request: TacticalRequest,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match tactical_request {
            TacticalRequest::Status => self.tactical_algorithm.status(),
            TacticalRequest::Scheduling => {
                todo!()
            }
            TacticalRequest::Resources => {
                todo!()
            }
            TacticalRequest::Days => {
                todo!()
            }
        }
    }
}

impl Handler<SetAddr> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAddr, _ctx: &mut Context<Self>) {
        match msg {
            SetAddr::SetSupervisor(id, addr) => {
                self.supervisor_addrs.insert(id, addr);
            }
            _ => {
                println!("The tactical agent received an Addr<T>, where T is not a valid Actor");
                todo!()
            }
        }
    }
}
