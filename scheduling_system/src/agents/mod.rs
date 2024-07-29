use actix::{Addr, Message};
use shared_types::scheduling_environment::{
    work_order::WorkOrderNumber, worker_environment::resources::Id,
};

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

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

/// The StateLink is a generic type that each type of Agent will implement.
/// The generics mean:
///     S: Strategic
///     T: Tactical
///     Su: Supervisor
///     O: Operational
/// This means that each Agent in the system will need to implement how to
/// understand messages from the other Agents in their own unique way.
/// This allows us to get custom implementations for each of the
/// Agent types creating a mesh of communication pathways that are still
/// statically typed.
pub enum StateLink<S, T, Su, O> {
    // This one is for the Strategic -> Tactical
    // Strategic(Vec<(WorkOrderNumber, Period)>),
    Strategic(S),
    // This one is for the Tactical -> Supervisor
    // Tactical(Vec<(WorkOrderNumber, HashMap<ActivityNumber, OperationSolution>)>),
    Tactical(T),
    // This one is for the Supervisor -> Operational
    // Supervisor(Delegate),
    Supervisor(Su),
    // This one is backwards and is for the Operational -> Supervisor
    // Operational(((Id, WorkOrderNumber, ActivityNumber), OperationalObjective)),
    Operational(O),
}

impl<S, T, Su, O> Message for StateLink<S, T, Su, O> {
    type Result = Result<(), StateLinkError>;
}

pub struct StateLinkError;

pub enum EnteringState<T> {
    Present,
    New(T),
    Obselete(T),
}

#[derive(Clone)]
pub struct UpdateWorkOrderMessage(pub WorkOrderNumber);

impl Message for UpdateWorkOrderMessage {
    type Result = ();
}
