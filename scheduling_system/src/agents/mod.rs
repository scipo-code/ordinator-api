pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::HashMap;

use actix::{Addr, Message};
use arc_swap::ArcSwap;
use shared_types::agent_error::AgentError;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
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

#[derive(Default)]
pub struct StrategicTacticalSolutionArcSwap(pub ArcSwap<SharedSolution>);

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SharedSolution {
    strategic: StrategicSolution,
    pub tactical: TacticalSolution,
}

// Should you delete these? I think that we should
pub trait StrategicInteration {
    fn strategic_scheduled_periods(&self) -> &HashMap<WorkOrderNumber, Option<Period>>;
    fn strategic_scheduled_periods_mut(&mut self) -> &mut HashMap<WorkOrderNumber, Option<Period>>;
    fn set_strategic_period(&mut self, work_order_number: WorkOrderNumber, period: Period);
}

impl StrategicInteration for SharedSolution {
    fn strategic_scheduled_periods(&self) -> &HashMap<WorkOrderNumber, Option<Period>> {
        &self.strategic.scheduled_periods
    }

    fn strategic_scheduled_periods_mut(&mut self) -> &mut HashMap<WorkOrderNumber, Option<Period>> {
        &mut self.strategic.scheduled_periods
    }

    fn set_strategic_period(&mut self, work_order_number: WorkOrderNumber, period: Period) {
        let previous_period = self
            .strategic
            .scheduled_periods
            .insert(work_order_number, Some(period));
        // TODO: Make assert here
        // assert!(previous)
    }
}

pub trait TacticalInteraction {
    fn tactical_period_mut(&mut self, work_order_number: &WorkOrderNumber) -> &mut Option<Period>;
    fn tactical_day(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> &Vec<(Day, Work)>;
    fn tactical_period(&self, work_order_number: &WorkOrderNumber) -> &Option<Period>;
    fn tactical_solution(
        &self,
        work_order_number: WorkOrderNumber,
    ) -> Option<HashMap<ActivityNumber, TacticalOperation>>;
}

impl SharedSolution {
    pub fn tactical_period_mut(
        &mut self,
        work_order_number: &WorkOrderNumber,
    ) -> &mut Option<Period> {
        self.tactical
            .scheduled_period
            .get_mut(work_order_number)
            .unwrap()
    }
    pub fn tactical_period(&self, work_order_number: &WorkOrderNumber) -> &Option<Period> {
        self.tactical
            .scheduled_period
            .get(work_order_number)
            .unwrap()
    }

    pub fn tactical_day(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> &Vec<(Day, Work)> {
        &self
            .tactical
            .tactical_solution
            .get(&work_order_number)
            .unwrap()
            .as_ref()
            .unwrap()
            .get(&activity_number)
            .unwrap()
            .scheduled
    }

    pub fn tactical_solution(
        &self,
        work_order_number: WorkOrderNumber,
    ) -> &Option<HashMap<ActivityNumber, TacticalOperation>> {
        self.tactical
            .tactical_solution
            .get(&work_order_number)
            .ok_or(AgentError::TacticalMissingState)
            .unwrap()
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StrategicSolution {
    pub scheduled_periods: HashMap<WorkOrderNumber, Option<Period>>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalSolution {
    pub tactical_solution:
        HashMap<WorkOrderNumber, Option<HashMap<ActivityNumber, TacticalOperation>>>,
    pub scheduled_period: HashMap<WorkOrderNumber, Option<Period>>,
}

impl TacticalSolution {
    pub fn tactical_remove_work_order(&mut self, work_order_number: &WorkOrderNumber) {
        self.tactical_solution.remove(work_order_number);
        self.scheduled_period.remove(work_order_number);
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
    type Result = Result<(), StateLinkError>;
}

#[allow(dead_code)]
pub struct StateLinkError(Option<Id>, Option<WorkOrderActivity>);

#[derive(Clone)]
pub struct UpdateWorkOrderMessage(pub WorkOrderNumber);

impl Message for UpdateWorkOrderMessage {
    type Result = ();
}
