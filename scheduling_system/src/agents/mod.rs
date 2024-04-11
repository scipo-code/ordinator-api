use std::fmt::Display;

use actix::{Addr, Message};
use shared_messages::resources::Id;

use crate::models::time_environment::period::Period;

use self::{
    operational_agent::OperationalAgent, strategic_agent::StrategicAgent,
    supervisor_agent::SupervisorAgent, tactical_agent::TacticalAgent,
};

pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;



#[allow(dead_code)]
pub enum SetAddr {
    Strategic(Addr<StrategicAgent>),
    Tactical(Addr<TacticalAgent>),
    Supervisor(Id, Addr<SupervisorAgent>),
    Operational(Id, Addr<OperationalAgent>),
}

impl Message for SetAddr {
    type Result = ();
}

#[allow(dead_code)]
pub enum SendState {
    Strategic(Vec<(u32, Period)>),
    Tactical,
    Supervisor,
    Operational,
}

impl Message for SendState {
    type Result = ();
}
