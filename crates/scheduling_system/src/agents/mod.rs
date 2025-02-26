pub mod operational_agent;
pub mod orchestrator;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Display};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use arc_swap::{ArcSwap, Guard};
use colored::Colorize;
use itertools::Itertools;
use operational_agent::algorithm::operational_parameter::OperationalParameters;
use operational_agent::algorithm::operational_solution::{
    Assignment, MarginalFitness, OperationalAssignment,
};
use operational_agent::algorithm::{OperationalObjectiveValue, Unavailability};
use orchestrator::NotifyOrchestrator;
use shared_types::agents::strategic::{OperationalResource, StrategicResources};
use shared_types::agents::supervisor::SupervisorObjectiveValue;
use shared_types::agents::tactical::{Days, TacticalObjectiveValue, TacticalResources};
use shared_types::orchestrator::ApiSolution;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::{ActivityNumber, Work};
use shared_types::scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
use shared_types::scheduling_environment::worker_environment::resources::{Id, Resources};
use shared_types::scheduling_environment::SchedulingEnvironment;
use strategic_agent::algorithm::strategic_parameters::StrategicParameters;
use strategic_agent::StrategicObjectiveValue;
use supervisor_agent::algorithm::delegate::Delegate;
use supervisor_agent::algorithm::supervisor_parameters::SupervisorParameters;
use tactical_agent::algorithm::tactical_parameters::TacticalParameters;
use tactical_agent::algorithm::tactical_solution::OperationSolution;
use traits::ActorBasedLargeNeighborhoodSearch;

pub struct Agent<Algorithm, AgentRequest, AgentResponse>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    agent_id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm,
    pub receiver_from_orchestrator: Receiver<AgentMessage<AgentRequest>>,
    pub sender_to_orchestrator: Sender<Result<AgentResponse>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl<Algorithm, AgentRequest, AgentResponse> Agent<Algorithm, AgentRequest, AgentResponse>
where
    Self: MessageHandler<Req = AgentRequest, Res = AgentResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    AgentRequest: Send + Sync + 'static,
    AgentResponse: Send + Sync + 'static,
{
    pub fn new(
        agent_id: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        algorithm: Algorithm,
        receiver_from_orchestrator: Receiver<AgentMessage<AgentRequest>>,
        sender_to_orchestrator: Sender<Result<AgentResponse>>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Self {
        Self {
            agent_id,
            scheduling_environment,
            algorithm,
            receiver_from_orchestrator,
            sender_to_orchestrator,
            notify_orchestrator,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut schedule_iteration = ScheduleIteration::default();

        self.algorithm
            .schedule()
            .with_context(|| {
                format!(
                    "Could not perform initial schedule iteration\nfile: {}\nline: {}",
                    file!(),
                    line!()
                )
            })
            .unwrap();

        schedule_iteration.increment();
        loop {
            while let Ok(message) = self.receiver_from_orchestrator.try_recv() {
                self.handle(message).unwrap();
            }

            self.algorithm
                .run_lns_iteration()
                .with_context(|| format!("{:#?}", schedule_iteration))
                .unwrap();

            schedule_iteration.increment();
        }
    }
}

pub struct Algorithm<S, P, I> {
    id: Id,
    solution_intermediate: I,
    solution: S,
    parameters: P,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    loaded_shared_solution: Guard<Arc<SharedSolution>>,
}

impl<S, P, I> AlgorithmUtils for Algorithm<S, P, I>
where
    I: Default,
    S: Solution + Debug + Clone,
{
    type Sol = S;
    type ObjectiveValue = S::ObjectiveValue;
    type Parameters = P;

    fn new(
        id: &Id,
        solution: S,
        parameters: P,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    ) -> Self {
        let loaded_shared_solution = arc_swap_shared_solution.0.load();

        Self {
            id: id.clone(),
            solution_intermediate: I::default(),
            solution,
            parameters,
            arc_swap_shared_solution,
            loaded_shared_solution,
        }
    }

    fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    fn clone_algorithm_solution(&self) -> S {
        self.solution.clone()
    }

    fn swap_solution(&mut self, solution: S) {
        self.solution = solution;
    }
}

#[derive(Default)]
pub struct ScheduleIteration {
    loop_iteration: u64,
}

impl ScheduleIteration {
    pub fn increment(&mut self) {
        self.loop_iteration += 1;
    }
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

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct SharedSolution {
    pub strategic: StrategicSolution,
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

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct StrategicSolution {
    pub objective_value: StrategicObjectiveValue,
    pub strategic_scheduled_work_orders: HashMap<WorkOrderNumber, Option<Period>>,
    pub strategic_loadings: StrategicResources,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalSolution {
    pub objective_value: TacticalObjectiveValue,
    pub tactical_work_orders: TacticalScheduledWorkOrders,
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
        self.0.tactical_work_orders.0 = tactical_days;
        self
    }

    pub fn build(self) -> TacticalSolution {
        TacticalSolution {
            objective_value: self.0.objective_value,
            tactical_work_orders: self.0.tactical_work_orders,
            tactical_loadings: self.0.tactical_loadings,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SupervisorSolution {
    pub objective_value: SupervisorObjectiveValue,
    operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct OperationalSolution {
    pub objective_value: OperationalObjectiveValue,
    pub scheduled_work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>,
}

impl Solution for StrategicSolution {
    type ObjectiveValue = StrategicObjectiveValue;
    type Parameters = StrategicParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        let strategic_loadings = parameters
            .strategic_capacity
            .0
            .iter()
            .map(|(per, res)| {
                let inner_map: HashMap<_, _> = res
                    .iter()
                    .map(|(id, or)| {
                        (
                            id.clone(),
                            OperationalResource::new(
                                id,
                                Work::from(0.0),
                                or.skill_hours.keys().cloned().collect_vec(),
                            ),
                        )
                    })
                    .collect();

                (per.clone(), inner_map)
            })
            .collect::<HashMap<_, _>>();

        let strategic_loadings = StrategicResources::new(strategic_loadings);

        let strategic_scheduled_work_orders = parameters
            .strategic_work_order_parameters
            .keys()
            .map(|won| (*won, None))
            .collect();

        let strategic_objective_value = StrategicObjectiveValue::new(&parameters.strategic_options);
        Self {
            objective_value: strategic_objective_value,
            strategic_scheduled_work_orders,
            strategic_loadings,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}
impl Solution for TacticalSolution {
    type ObjectiveValue = TacticalObjectiveValue;
    type Parameters = TacticalParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        let tactical_loadings_inner: HashMap<Resources, Days> = parameters
            .tactical_capacity
            .resources
            .iter()
            .map(|(wo, days)| {
                let inner_map = days
                    .days
                    .keys()
                    .map(|day| (day.clone(), Work::from(0.0)))
                    .collect();
                (*wo, Days::new(inner_map))
            })
            .collect();

        let tactical_scheduled_work_orders_inner: HashMap<_, _> = parameters
            .tactical_work_orders
            .keys()
            .map(|won| (*won, WhereIsWorkOrder::NotScheduled))
            .collect();

        Self {
            objective_value: TacticalObjectiveValue::default(),
            tactical_work_orders: TacticalScheduledWorkOrders(tactical_scheduled_work_orders_inner),
            tactical_loadings: TacticalResources::new(tactical_loadings_inner),
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}
impl Solution for SupervisorSolution {
    type ObjectiveValue = SupervisorObjectiveValue;
    type Parameters = SupervisorParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        // The SupervisorParameters should have knowledge of the agents.

        let operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate> = parameters
            .supervisor_work_orders
            .iter()
            .flat_map(|(won, inner)| {
                inner.iter().flat_map(|(acn, sp)| {
                    parameters
                        .operational_ids
                        .iter()
                        .filter(|e| e.1.contains(&sp.resource))
                        .map(|e| ((e.clone(), (*won, *acn)), Delegate::Assess))
                })
            })
            .collect();

        let objective_value = Self::ObjectiveValue::default();

        Self {
            objective_value,
            operational_state_machine,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}
impl Solution for OperationalSolution {
    type ObjectiveValue = OperationalObjectiveValue;
    type Parameters = OperationalParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        let mut scheduled_work_order_activities = Vec::new();

        let start_event =
            Assignment::make_unavailable_event(Unavailability::Beginning, &parameters.availability);

        let end_event =
            Assignment::make_unavailable_event(Unavailability::End, &parameters.availability);

        let unavailability_start_event = OperationalAssignment::new(vec![start_event]);

        let unavailability_end_event = OperationalAssignment::new(vec![end_event]);

        scheduled_work_order_activities.push((
            (WorkOrderNumber(0), ActivityNumber(0)),
            unavailability_start_event,
        ));

        scheduled_work_order_activities.push((
            (WorkOrderNumber(0), ActivityNumber(0)),
            unavailability_end_event,
        ));

        Self {
            objective_value: 0,
            scheduled_work_order_activities,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}

impl StrategicSolution {
    pub fn supervisor_work_orders_from_strategic(
        &self,
        supervisor_periods: &[Period],
    ) -> HashSet<WorkOrderNumber> {
        let mut supervisor_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

        self.strategic_scheduled_work_orders
            .iter()
            .for_each(|(won, opt_per)| {
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
    pub fn delegates_for_agent(
        &self,
        operational_agent: &Id,
    ) -> HashMap<WorkOrderActivity, Delegate> {
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
        for delegate in self.delegates_for_agent(operational_agent).values() {
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
}

// Should the new function take in the `parameters` as an function parameter?
impl TacticalSolution {
    pub fn release_from_tactical_solution(&mut self, work_order_number: &WorkOrderNumber) {
        self.tactical_work_orders
            .0
            .insert(*work_order_number, WhereIsWorkOrder::Strategic);
    }
    pub fn tactical_scheduled_days(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> Result<&Vec<(Day, Work)>> {
        let tactical_day = &self
            .tactical_work_orders
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
        self.tactical_work_orders.0.insert(
            work_order_number,
            WhereIsWorkOrder::Tactical(tactical_scheduled_operations),
        );
    }
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
            .scheduled_work_order_activities
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
// FIX
// This could be generic! I think that it should.
impl<Algorithm, AgentRequest, ResponseMessage> Agent<Algorithm, AgentRequest, ResponseMessage>
where
    Self: MessageHandler<Req = AgentRequest, Res = ResponseMessage>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ResponseMessage: Sync + Send + 'static,
{
    pub fn handle(&mut self, agent_message: AgentMessage<AgentRequest>) -> Result<()> {
        match agent_message {
            AgentMessage::State(state_link) => self.handle_state_link(state_link)?,
            AgentMessage::Actor(strategic_request_message) => {
                let message = self.handle_request_message(strategic_request_message);

                self.sender_to_orchestrator.send(message)?;
            }
        }
        Ok(())
    }
}

/// This type is the primary message type that all agents should receive.
/// All agents should have the `StateLink` and each agent then have its own
/// ActorRequest which is specifically created for each agent.
#[derive(Clone)]
pub enum AgentMessage<ActorRequest> {
    State(StateLink),
    Actor(ActorRequest),
    // FIX
    // Add Options here so that every agent can have its options updated at run time.
    // Options(),
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
