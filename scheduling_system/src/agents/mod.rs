use actix::{Addr, Message};
use shared_messages::resources::Id;

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

pub enum SetAddr {
    SetStrategic(Addr<StrategicAgent>),
    SetTactical(Addr<TacticalAgent>),
    SetSupervisor(Id, Addr<SupervisorAgent>),
    SetOperational(Id, Addr<OperationalAgent>),
}

impl Message for SetAddr {
    type Result = ();
}
