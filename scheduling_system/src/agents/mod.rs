use std::collections::HashMap;

use actix::{Addr, Message};
use operational_agent::algorithm::OperationalObjective;
use shared_messages::scheduling_environment::work_order::operation::ActivityNumber;
use shared_messages::scheduling_environment::{
    work_order::WorkOrderNumber, worker_environment::resources::Id,
};

use shared_messages::scheduling_environment::time_environment::period::Period;

use self::supervisor_agent::Delegate;
use self::{
    operational_agent::OperationalAgent,
    strategic_agent::StrategicAgent,
    supervisor_agent::SupervisorAgent,
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
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
pub enum StateLink {
    Strategic(Vec<(WorkOrderNumber, Period)>),
    Tactical(Vec<(WorkOrderNumber, HashMap<ActivityNumber, OperationSolution>)>),
    Supervisor(Delegate),
    Operational(((Id, WorkOrderNumber, ActivityNumber), OperationalObjective)),
}

impl Message for StateLink {
    type Result = ();
}
#[derive(Clone)]
pub struct UpdateWorkOrderMessage(pub WorkOrderNumber);

impl Message for UpdateWorkOrderMessage {
    type Result = ();
}
