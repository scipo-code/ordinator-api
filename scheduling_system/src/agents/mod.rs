pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use arc_swap::ArcSwap;
use colored::Colorize;
use operational_agent::algorithm::operational_solution::{MarginalFitness, OperationalAssignment};
use operational_agent::algorithm::OperationalObjectiveValue;
use orchestrator::NotifyOrchestrator;
use shared_types::orchestrator::ApiSolution;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::scheduling_environment::SchedulingEnvironment;
use shared_types::strategic::{StrategicObjectiveValue, StrategicResources};
use shared_types::tactical::{TacticalObjectiveValue, TacticalResources};
use shared_types::Asset;
use supervisor_agent::algorithm::delegate::Delegate;
use tactical_agent::algorithm::tactical_solution::OperationSolution;
use traits::{ActorBasedLargeNeighborhoodSearch, Solution};

pub struct Agent<Algorithm, AgentRequest, AgentResponse>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    asset: Asset,
    agent_id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm,
    pub receiver_from_orchestrator: Receiver<AgentMessage<AgentRequest>>,
    pub sender_to_orchestrator: Sender<Result<AgentResponse>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl<Algorithm, AgentRequest, AgentResponse> Agent<Algorithm, AgentRequest, AgentResponse>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    pub fn new(
        asset: Asset,
        agent_id: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        algorithm: Algorithm,
        receiver_from_orchestrator: Receiver<AgentMessage<AgentRequest>>,
        sender_to_orchestrator: Sender<Result<AgentResponse>>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Self {
        Self {
            asset,
            agent_id,
            scheduling_environment,
            algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        }
    }
}

#[derive(Default)]
pub struct ScheduleIteration {
    loop_iteration: u64,
}

impl fmt::Debug for ScheduleIteration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let string = format!(
                "{}: {}",
                std::any::type_name::<ScheduleIteration>()
                    .split("::")
                    .last()
                    .unwrap(),
                self.loop_iteration
            )
            .bright_magenta();

            write!(f, "{}", string)
        } else {
            f.debug_struct("ScheduleIteration")
                .field("loop_iteration", &self.loop_iteration)
                .finish()
        }
    }
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

impl From<SharedSolution> for ApiSolution {
    fn from(_value: SharedSolution) -> Self {
        ApiSolution {
            strategic: "NEEDS TO BE IMPLEMENTED".to_string(),
            tactical: "NEEDS TO BE IMPLEMENTED".to_string(),
            supervisor: "NEEDS TO BE IMPLEMENTED".to_string(),
            operational: "NEEDS TO BE IMPLEMENTED".to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StrategicSolution {
    pub objective_value: StrategicObjectiveValue,
    pub strategic_periods: HashMap<WorkOrderNumber, Option<Period>>,
    pub strategic_loadings: StrategicResources,
}

impl Solution for StrategicSolution {}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalSolution {
    pub objective_value: TacticalObjectiveValue,
    pub tactical_scheduled_work_orders: TacticalScheduledWorkOrders,
    pub tactical_loadings: TacticalResources,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalScheduledWorkOrders(
    pub HashMap<WorkOrderNumber, WhereIsWorkOrder<TacticalScheduledOperations>>,
);

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub enum WhereIsWorkOrder<T> {
    Strategic,
    Tactical(T),
    #[default]
    NotScheduled,
}
impl WhereIsWorkOrder<TacticalScheduledOperations> {
    pub fn is_tactical(&self) -> bool {
        matches!(self, WhereIsWorkOrder::Tactical(_))
    }

    pub fn tactical_operations(&self) -> Result<&TacticalScheduledOperations> {
        match self {
            WhereIsWorkOrder::Strategic => bail!(
                "A call to extract the {} was made but received {}",
                std::any::type_name::<TacticalScheduledOperations>(),
                std::any::type_name_of_val(self),
            ),
            WhereIsWorkOrder::Tactical(tactical_scheduled_operations) => {
                Ok(tactical_scheduled_operations)
            }
            WhereIsWorkOrder::NotScheduled => bail!(
                "The work order has not been scheduled yet, you are most likely calling this method before complete initialization"
            ),
        }
    }
}

impl TacticalScheduledWorkOrders {
    pub fn scheduled_work_orders(&self) -> usize {
        self.0
            .iter()
            .filter(|(_won, sch_wo)| sch_wo.is_tactical())
            .count()
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalScheduledOperations(pub HashMap<ActivityNumber, OperationSolution>);

impl TacticalScheduledOperations {
    fn insert_operation_solution(
        &mut self,
        activity: ActivityNumber,
        operation_solution: OperationSolution,
    ) {
        self.0.insert(activity, operation_solution);
    }
}

impl Display for TacticalScheduledOperations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tactical_operations = self.0.iter().collect::<Vec<_>>();
        tactical_operations
            .sort_by(|a, b| a.1.work_order_activity.1.cmp(&b.1.work_order_activity.1));

        for operation_solution in tactical_operations {
            write!(f, "activity: {:#?}", operation_solution.0)?;
            write!(f, "{}", operation_solution.1)?;
        }
        Ok(())
    }
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
        tactical_days: HashMap<WorkOrderNumber, WhereIsWorkOrder<TacticalScheduledOperations>>,
    ) -> Self {
        self.0.tactical_scheduled_work_orders.0 = tactical_days;
        self
    }

    pub fn build(self) -> TacticalSolution {
        TacticalSolution {
            objective_value: self.0.objective_value,
            tactical_scheduled_work_orders: self.0.tactical_scheduled_work_orders,
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
    pub work_order_activities_assignment: Vec<(WorkOrderActivity, OperationalAssignment)>,
}

impl Solution for OperationalSolution {}

impl StrategicSolution {
    pub fn supervisor_work_orders_from_strategic(
        &self,
        supervisor_periods: &[Period],
    ) -> HashSet<WorkOrderNumber> {
        let mut supervisor_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

        self.strategic_periods.iter().for_each(|(won, opt_per)| {
            if let Some(period) = opt_per {
                if supervisor_periods.contains(period) {
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
        for delegate in self.state_of_agent(operational_agent).values() {
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
    pub fn release_from_tactical_solution(&mut self, work_order_number: &WorkOrderNumber) {
        self.tactical_scheduled_work_orders
            .0
            .insert(*work_order_number, WhereIsWorkOrder::Strategic);
    }
    pub fn tactical_scheduled_days(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> Result<&Vec<(Day, Work)>> {
        let tactical_day = &self
            .tactical_scheduled_work_orders
            .0
            .get(work_order_number)
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not present in the tactical solution",
                    work_order_number
                )
            })?
            .tactical_operations()
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not scheduled for the tactical solution",
                    work_order_number
                )
            })?
            .0
            .get(activity_number)
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
        tactical_scheduled_operations: TacticalScheduledOperations,
    ) {
        self.tactical_scheduled_work_orders.0.insert(
            work_order_number,
            WhereIsWorkOrder::Tactical(tactical_scheduled_operations),
        );
    }
}

#[allow(dead_code)]
pub trait GetMarginalFitness {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness>;
}

impl GetMarginalFitness for HashMap<Id, OperationalSolution> {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness> {
        self.get(operational_agent)
            .with_context(|| {
                format!(
                    "Could not find {} for operational agent: {:#?}",
                    std::any::type_name::<MarginalFitness>(),
                    operational_agent,
                )
            })?
            .work_order_activities_assignment
            .iter()
            .find(|woa_os| woa_os.0 == *work_order_activity)
            .map(|os| &os.1.marginal_fitness)
            .with_context(|| {
                format!(
                    "{} did not have\n{:#?}",
                    operational_agent.to_string().bright_blue(),
                    format!("{:#?}", work_order_activity).bright_yellow()
                )
            })
    }
}

/// This type is the primary message type that all agents should receive.
/// All agents should have the `StateLink` and each agent then have its own
/// ActorRequest which is specifically created for each agent.
#[derive(Clone)]
pub enum AgentMessage<ActorRequest> {
    State(StateLink),
    Actor(ActorRequest),
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
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StateLink {
    WorkOrders(AgentSpecific),
    WorkerEnvironment,
    TimeEnvironment,
}

#[derive(Debug, Clone)]
pub enum AgentSpecific {
    Strategic(Vec<WorkOrderNumber>),
}
