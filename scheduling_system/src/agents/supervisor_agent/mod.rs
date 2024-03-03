use std::sync::{Arc, Mutex};

use actix::prelude::*;

use crate::models::SchedulingEnvironment;

pub struct SupervisorAgent {
    id: String,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

impl Actor for SupervisorAgent {
    type Context = Context<Self>;
}

impl SupervisorAgent {
    pub fn new(
        id: String,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> SupervisorAgent {
        SupervisorAgent {
            id,
            scheduling_environment,
        }
    }
}
