use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_messages::{
    agent_error::AgentError, resources::Id, supervisor::SupervisorRequestMessage, Asset,
    StatusMessage, StopMessage,
};
use tracing::instrument;

use crate::models::SchedulingEnvironment;

use super::{
    operational_agent::OperationalAgent,
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
    SetAddr, StateLink,
};

pub struct SupervisorAgent {
    id: Id,
    #[allow(dead_code)]
    asset: Asset,
    #[allow(dead_code)]
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    assigned_work_orders: Vec<(u32, HashMap<u32, OperationSolution>)>,
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
            .do_send(SetAddr::Supervisor(self.id.clone(), ctx.address()));
    }
}

impl SupervisorAgent {
    pub fn new(
        id: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> SupervisorAgent {
        SupervisorAgent {
            id,
            asset,
            scheduling_environment,
            assigned_work_orders: Vec::new(),
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
        }
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, Work Center: {:?}, Main Work Center: {:?}",
            self.id.0, self.id.1, self.id.2
        )
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
        if let SetAddr::Operational(id, addr) = msg {
            self.operational_agent_addrs.insert(id, addr);
        }
    }
}

impl Handler<StateLink> for SupervisorAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, msg: StateLink, _ctx: &mut Self::Context) {
        match msg {
            StateLink::Strategic(_) => {}
            StateLink::Tactical(_) => {}
            StateLink::Supervisor => {}
            StateLink::Operational => {}
        }
    }
}

impl Handler<SupervisorRequestMessage> for SupervisorAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "trace", skip_all)]
    fn handle(
        &mut self,
        supervisor_request_message: SupervisorRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        tracing::info!(
            "Received SupervisorRequestMessage: {:?}",
            supervisor_request_message
        );

        match supervisor_request_message {
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                Ok(format!(
                    "Received SupervisorStatusMessage: {:?}",
                    self.assigned_work_orders
                ))
            }
            SupervisorRequestMessage::Test => Ok("Test".to_string()),
        }
    }
}
