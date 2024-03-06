pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_messages::tactical::TacticalRequest;
use std::sync::{Arc, Mutex};

use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::models::SchedulingEnvironment;

#[allow(dead_code)]
pub struct TacticalAgent {
    id: i32,
    days: u32,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    tactical_algorithm: TacticalAlgorithm,
}

impl TacticalAgent {
    pub fn new(
        id: i32,
        days: u32,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        TacticalAgent {
            id,
            days,
            scheduling_environment,
            tactical_algorithm: TacticalAlgorithm::new(),
        }
    }

    pub fn get_time_horizon(&self) -> u32 {
        self.days
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("WorkPlannerAgent is alive and julia is running");
    }
}

impl Handler<TacticalRequest> for TacticalAgent {
    type Result = String;

    fn handle(
        &mut self,
        tactical_request: TacticalRequest,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        println!("WorkPlannerAgent received WorkPlannerMessage");
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
