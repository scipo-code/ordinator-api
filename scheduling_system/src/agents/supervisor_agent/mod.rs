use std::sync::{Arc, Mutex};

use actix::prelude::*;
use shared_messages::{resources::Id, StatusMessage, StopMessage};

use crate::models::SchedulingEnvironment;

pub struct SupervisorAgent {
    id: Id,
    resource: shared_messages::resources::Resources,
    #[allow(dead_code)]
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl Actor for SupervisorAgent {
    type Context = Context<Self>;
}

impl SupervisorAgent {
    pub fn new(
        id: Id,
        resource: shared_messages::resources::Resources,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> SupervisorAgent {
        SupervisorAgent {
            id,
            resource,
            scheduling_environment,
        }
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!("ID: {}, Work Center: {}", self.id, self.resource)
    }
}

impl Handler<StopMessage> for SupervisorAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
