pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::HashMap;

use actix::{Addr, Message};
use anyhow::Result;
use arc_swap::ArcSwap;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use tactical_agent::tactical_algorithm::TacticalOperation;
use tracing::Span;

use self::{
    operational_agent::OperationalAgent, strategic_agent::StrategicAgent,
    supervisor_agent::SupervisorAgent, tactical_agent::TacticalAgent,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct AssertError(String);

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct ScheduleIteration {}

#[allow(dead_code)]
pub enum SetAddr {
    Strategic(Addr<StrategicAgent>),
    Tactical(Addr<TacticalAgent>),
    Supervisor(Id, Addr<SupervisorAgent>),
    Operational(Id, Addr<OperationalAgent>),
}

impl Message for SetAddr {
    type Result = Result<()>;
}

#[derive(Default)]
pub struct ArcSwapSharedSolution(pub ArcSwap<SharedSolution>);

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SharedSolution {
    strategic: StrategicSolution,
    pub tactical: TacticalSolution,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StrategicSolution {
    pub scheduled_periods: HashMap<WorkOrderNumber, Option<Period>>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalSolution {
    pub tactical_days: HashMap<WorkOrderNumber, Option<HashMap<ActivityNumber, TacticalOperation>>>,
    pub tactical_period: HashMap<WorkOrderNumber, Option<Period>>,
}

impl TacticalSolution {
    pub fn tactical_remove_work_order(&mut self, work_order_number: &WorkOrderNumber) {
        self.tactical_days.remove(work_order_number);
        self.tactical_period.remove(work_order_number);
    }
    pub fn tactical_day(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> &Vec<(Day, Work)> {
        &self
            .tactical_days
            .get(&work_order_number)
            .unwrap()
            .as_ref()
            .unwrap()
            .get(&activity_number)
            .unwrap()
            .scheduled
    }
    pub fn tactical_period(&self, work_order_number: &WorkOrderNumber) -> &Option<Period> {
        self.tactical_period.get(work_order_number).unwrap()
    }
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
#[derive(Debug)]
pub struct StateLinkWrapper<S, T, Su, O> {
    state_link: StateLink<S, T, Su, O>,
    span: Span,
}

impl<S, T, Su, O> StateLinkWrapper<S, T, Su, O> {
    pub fn new(state_link: StateLink<S, T, Su, O>, span: Span) -> Self {
        Self { state_link, span }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StateLink<S, T, Su, O> {
    Strategic(S),
    Tactical(T),
    Supervisor(Su),
    Operational(O),
}

impl<S, T, Su, O> Message for StateLinkWrapper<S, T, Su, O> {
    type Result = Result<()>;
}

#[derive(Clone)]
pub struct UpdateWorkOrderMessage(pub WorkOrderNumber);

impl Message for UpdateWorkOrderMessage {
    type Result = ();
}
