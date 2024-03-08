use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_messages::{resources::Id, StatusMessage, StopMessage};
use tracing::instrument;

use crate::models::SchedulingEnvironment;

use super::{operational_agent::OperationalAgent, tactical_agent::TacticalAgent, SetAddr};

pub struct SupervisorAgent {
    id: Id,
    #[allow(dead_code)]
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    #[allow(dead_code)]
    tactical_agent_addr: Addr<TacticalAgent>,
    #[allow(dead_code)]
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

impl Actor for SupervisorAgent {
    type Context = Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.tactical_agent_addr
            .do_send(SetAddr::SetSupervisor(self.id.clone(), ctx.address()));
    }
}

impl SupervisorAgent {
    pub fn new(
        id: Id,
        tactical_agent_addr: Addr<TacticalAgent>,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> SupervisorAgent {
        SupervisorAgent {
            id,
            scheduling_environment,
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
        }
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!("ID: {}, Work Center: {:?}", self.id.0, self.id.1)
    }
}

impl Handler<StopMessage> for SupervisorAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}


impl Handler<SetAddr> for SupervisorAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, msg: SetAddr, _ctx: &mut Self::Context) {
        match msg {
            SetAddr::SetOperational(id, addr) => {
                self.operational_agent_addrs.insert(id, addr);
            }
            _ => {}
        }
    }
}