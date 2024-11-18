pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::{HashMap, HashSet};

use actix::{Addr, Message};
use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use operational_agent::algorithm::{OperationalAssignment, OperationalObjectiveValue};
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::strategic::{StrategicObjectiveValue, StrategicResources};
use shared_types::tactical::{TacticalObjectiveValue, TacticalResources};
use supervisor_agent::algorithm::MarginalFitness;
use supervisor_agent::delegate::Delegate;
use tactical_agent::tactical_algorithm::TacticalOperation;
use tracing::{event, Level, Span};

use self::{
    operational_agent::OperationalAgent, strategic_agent::StrategicAgent,
    supervisor_agent::SupervisorAgent, tactical_agent::TacticalAgent,
};

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct ScheduleIteration {}

#[allow(dead_code)]
pub enum SetAddr {
    Strategic(Addr<StrategicAgent>),
    Tactical(Addr<TacticalAgent>),
    Supervisor(String, Addr<SupervisorAgent>),
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
    pub supervisor: SupervisorSolution,
    pub operational: HashMap<Id, OperationalSolution>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StrategicSolution {
    pub objective_value: StrategicObjectiveValue,
    pub strategic_periods: HashMap<WorkOrderNumber, Option<Period>>,
    pub strategic_loadings: StrategicResources,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalSolution {
    pub objective_value: TacticalObjectiveValue,
    pub tactical_days: HashMap<WorkOrderNumber, Option<HashMap<ActivityNumber, TacticalOperation>>>,
    pub tactical_loadings: TacticalResources,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct TacticalSolutionBuilder(TacticalSolution);

#[allow(dead_code)]
impl TacticalSolutionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tactical_days(
        mut self,
        tactical_days: HashMap<WorkOrderNumber, Option<HashMap<ActivityNumber, TacticalOperation>>>,
    ) -> Self {
        self.0.tactical_days = tactical_days;
        self
    }

    pub fn build(self) -> TacticalSolution {
        TacticalSolution {
            objective_value: self.0.objective_value,
            tactical_days: self.0.tactical_days,
            tactical_loadings: self.0.tactical_loadings,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SupervisorSolution {
    objective_value: u64,
    operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct OperationalSolution {
    pub objective_value: OperationalObjectiveValue,
    pub work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>,
}

impl StrategicSolution {
    pub fn supervisor_work_orders_from_strategic(
        &self,
        supervisor_periods: &[Period],
    ) -> HashSet<WorkOrderNumber> {
        let mut supervisor_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

        self.strategic_periods.iter().for_each(|(won, opt_per)| {
            if let Some(period) = opt_per {
                if supervisor_periods.contains(period) {
                    event!(Level::WARN, period = ?period);
                    supervisor_work_orders.insert(*won);
                }
            }
        });
        supervisor_work_orders
    }
}

impl SupervisorSolution {
    pub fn state_of_agent(&self, operational_agent: &Id) -> HashMap<WorkOrderActivity, Delegate> {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| &id_woa.0 == operational_agent)
            .map(|(id_woa, del)| (id_woa.1, *del))
            .collect()
    }

    pub fn count_delegate_types(&self, operational_agent: &Id) -> (u64, u64, u64) {
        let mut count_assign = 0;
        let mut count_assess = 0;
        let mut count_unassign = 0;
        for (_, delegate) in &self.state_of_agent(operational_agent) {
            match delegate {
                Delegate::Assess => count_assess += 1,
                Delegate::Assign => count_assign += 1,
                Delegate::Unassign => count_unassign += 1,
                Delegate::Drop => (),
                Delegate::Done => (),
                Delegate::Fixed => (),
            }
        }
        (count_assign, count_assess, count_unassign)
    }

    fn remove_leaving_work_order_activities(
        &mut self,
        entering_work_orders_from_strategic: &HashSet<WorkOrderNumber>,
    ) {
        let supervisor_work_orders: HashSet<WorkOrderNumber> = self
            .operational_state_machine
            .iter()
            .map(|((_, woa), _)| woa.0)
            .collect();

        let leaving: HashSet<_> = supervisor_work_orders
            .difference(entering_work_orders_from_strategic)
            .collect();

        self.operational_state_machine
            .retain(|(_, woa), _| !leaving.contains(&woa.0));
    }
}

impl TacticalSolution {
    pub fn tactical_remove_work_order(&mut self, work_order_number: &WorkOrderNumber) {
        *self
            .tactical_days
            .get_mut(work_order_number)
            .expect("Tacical State is wrong") = None;
    }
    pub fn tactical_day(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> Result<&Vec<(Day, Work)>> {
        let tactical_day = &self
            .tactical_days
            .get(&work_order_number)
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not present in the tactical solution",
                    work_order_number
                )
            })?
            .as_ref()
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not scheduled for the tactical solution",
                    work_order_number
                )
            })?
            .get(&activity_number)
            .with_context(|| {
                format!(
                    "ActivityNumber: {:?} was not present in the tactical solution",
                    activity_number
                )
            })?
            .scheduled;

        Ok(tactical_day)
    }

    fn tactical_insert_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        tactical_days: HashMap<ActivityNumber, TacticalOperation>,
    ) {
        self.tactical_days
            .insert(work_order_number, Some(tactical_days));
    }
}

pub trait GetMarginalFitness {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<MarginalFitness>;
}

impl GetMarginalFitness for HashMap<Id, OperationalSolution> {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<MarginalFitness> {
        let marginal_fitness = &self
            .get(operational_agent)
            .with_context(|| {
                format!(
                    "Could not find <Auxiliary Objective> for operational agent: {:?} on {:?}",
                    operational_agent, work_order_activity
                )
            })?
            .work_order_activities
            .iter()
            .find(|ele| ele.0 == *work_order_activity)
            .map(|os| os.1.marginal_fitness.clone())
            .unwrap_or(MarginalFitness::MAX);

        Ok(marginal_fitness.clone())
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

impl<S, T, Su, O> StateLinkWrapper<S, T, Su, O> {}

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
