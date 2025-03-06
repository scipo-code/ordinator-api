pub mod operational_agent;
pub mod strategic_agent;
pub mod supervisor_agent;
pub mod tactical_agent;
pub mod traits;

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Display};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};

use anyhow::{bail, Context, Result};
use arc_swap::{ArcSwap, Guard};
use colored::Colorize;
use itertools::Itertools;

use traits::{
    ActorBasedLargeNeighborhoodSearch, GetMarginalFitness, MessageHandler, Parameters, Solution,
};

use super::orchestrator::NotifyOrchestrator;
use operational_agent::algorithm::operational_parameter::OperationalParameters;
use operational_agent::algorithm::operational_solution::Assignment;
use operational_agent::algorithm::operational_solution::MarginalFitness;
use operational_agent::algorithm::operational_solution::OperationalAssignment;
use operational_agent::algorithm::{OperationalObjectiveValue, Unavailability};
use shared_types::agents::strategic::{
    OperationalResource, StrategicRequestMessage, StrategicResources, StrategicResponseMessage,
};
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

use crate::orchestrator::agent_registry::Communication;
use crate::orchestrator::configuration::SystemConfigurations;

// TODO [ ] FIX [ ]
// You should reuse the trait bounds on the Agent and the Algorithm.
pub struct Agent<AgentRequest, AgentResponse, S, P, I>
where
    Algorithm<S, P, I>: ActorBasedLargeNeighborhoodSearch,
    S: Solution,
    P: Parameters,
{
    agent_id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm<S, P, I>,
    pub receiver_from_orchestrator: Receiver<ActorMessage<AgentRequest>>,
    pub sender_to_orchestrator: Sender<Result<AgentResponse>>,
    pub configurations: Arc<RwLock<SystemConfigurations>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl<ActorRequest, ActorResponse, S, P, I> Agent<ActorRequest, ActorResponse, S, P, I>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I>: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone,
    P: Parameters,
    I: Default,
{
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

    pub fn builder() -> AgentBuilder<ActorRequest, ActorResponse, S, P, I> {
        AgentBuilder {
            agent_id: None,
            scheduling_environment: None,
            algorithm: None,
            receiver_from_orchestrator: None,
            sender_to_orchestrator: None,
            configurations: None,
            notify_orchestrator: None,
            communication_for_orchestrator: None,
        }
    }
}

pub struct AgentBuilder<ActorRequest, ActorResponse, S, P, I>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I>: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone,
    P: Parameters,
    I: Default,
{
    agent_id: Option<Id>,
    scheduling_environment: Option<Arc<Mutex<SchedulingEnvironment>>>,
    algorithm: Option<Algorithm<S, P, I>>,
    receiver_from_orchestrator: Option<Receiver<ActorMessage<ActorRequest>>>,
    sender_to_orchestrator: Option<Sender<Result<ActorResponse>>>,
    configurations: Option<Arc<RwLock<SystemConfigurations>>>,
    notify_orchestrator: Option<NotifyOrchestrator>,
    //
    communication_for_orchestrator:
        Option<Communication<ActorMessage<ActorRequest>, ActorResponse>>,
}

impl<ActorRequest, ActorResponse, S, P, I> AgentBuilder<ActorRequest, ActorResponse, S, P, I>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I>: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone,
    P: Parameters,
    I: Default,
{
    pub fn build(self) -> Communication<ActorMessage<ActorRequest>, ActorResponse> {
        let agent = Agent {
            agent_id: self.agent_id.unwrap(),
            scheduling_environment: self.scheduling_environment.unwrap(),
            algorithm: self.algorithm.unwrap(),
            receiver_from_orchestrator: self.receiver_from_orchestrator.unwrap(),
            sender_to_orchestrator: self.sender_to_orchestrator.unwrap(),
            configurations: self.configurations.unwrap(),
            notify_orchestrator: self.notify_orchestrator.unwrap(),
        };
        let thread_name = format!(
            "{} for Asset: {}",
            std::any::type_name_of_val(&agent),
            agent
                .agent_id
                .2
                .first()
                .expect("Every agent needs to be associated with an Asset"),
        );
        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || agent.run())?;

        self.communication_for_orchestrator.unwrap()
    }

    pub fn agent_id(mut self, agent_id: Id) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
    pub fn scheduling_environment(
        mut self,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        self.scheduling_environment = Some(scheduling_environment);
        self
    }

    pub fn algorithm<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(AlgorithmBuilder<S, P, I>) -> AlgorithmBuilder<S, P, I>,
    {
        let algorithm_builder = Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder);

        self.algorithm = Some(algorithm_builder.build());
        self
    }

    pub fn communication(mut self) -> Self {
        let (sender_to_agent, receiver_from_orchestrator): (
            std::sync::mpsc::Sender<ActorMessage<ActorRequest>>,
            std::sync::mpsc::Receiver<ActorMessage<ActorRequest>>,
        ) = std::sync::mpsc::channel();

        let (sender_to_orchestrator, receiver_from_agent): (
            std::sync::mpsc::Sender<std::result::Result<ActorResponse, anyhow::Error>>,
            std::sync::mpsc::Receiver<std::result::Result<ActorResponse, anyhow::Error>>,
        ) = std::sync::mpsc::channel();

        self.communication_for_orchestrator = Some(Communication {
            sender: sender_to_agent,
            receiver: receiver_from_agent,
        });

        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self
    }
    pub fn receiver_from_orchestrator(
        mut self,
        receiver_from_orchestrator: Receiver<ActorMessage<ActorRequest>>,
    ) -> Self {
        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self
    }
    pub fn sender_to_orchestrator(
        mut self,
        sender_to_orchestrator: Sender<Result<ActorResponse>>,
    ) -> Self {
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self
    }
    pub fn configurations(mut self, configurations: Arc<RwLock<SystemConfigurations>>) -> Self {
        self.configurations = Some(configurations);
        self
    }
    pub fn notify_orchestrator(mut self, notify_orchestrator: NotifyOrchestrator) -> Self {
        self.notify_orchestrator = Some(notify_orchestrator);
        self
    }
}

pub struct Algorithm<S, P, I>
where
    S: Solution,
    P: Parameters,
{
    id: Id,
    solution_intermediate: I,
    solution: S,
    parameters: P,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    loaded_shared_solution: Guard<Arc<SharedSolution>>,
}

pub struct AlgorithmBuilder<S, P, I>
where
    S: Solution,
    P: Parameters,
{
    id: Option<Id>,
    solution_intermediate: Option<I>,
    solution: Option<S>,
    parameters: Option<P>,
    arc_swap_shared_solution: Option<Arc<ArcSwapSharedSolution>>,
    loaded_shared_solution: Option<Guard<Arc<SharedSolution>>>,
}

impl<S, P, I> Algorithm<S, P, I>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
{
    fn builder() -> AlgorithmBuilder<S, P, I> {
        AlgorithmBuilder {
            id: None,
            solution_intermediate: None,
            solution: None,
            parameters: None,
            arc_swap_shared_solution: None,
            loaded_shared_solution: None,
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

impl<S, P, I> AlgorithmBuilder<S, P, I>
where
    S: Solution,
    P: Parameters,
    I: Default,
{
    pub fn build(self) -> Algorithm<S, P, I> {
        Algorithm {
            id: self.id.unwrap(),
            solution_intermediate: self.solution_intermediate.unwrap(),
            solution: self.solution.unwrap(),
            parameters: self.parameters.unwrap(),
            arc_swap_shared_solution: self.arc_swap_shared_solution.unwrap(),
            loaded_shared_solution: self.loaded_shared_solution.unwrap(),
        }
    }
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
    pub fn solution_intermediate(&mut self, solution_intermediate: I) -> &mut Self {
        self.solution_intermediate = Some(solution_intermediate);
        self
    }
    // This should call the relevant method instead of the
    pub fn solution<F>(mut self, f: F) -> Self
    where
        F: FnOnce(S::Builder) -> S::Builder,
    {
        let solution_builder = S::builder();
        let solution_builder = f(solution_builder);
        self.solution = Some(solution_builder.build());
        self
    }

    pub fn parameters(mut self, parameters: P) -> Self {
        self.parameters = Some(parameters);
        self
    }
    pub fn arc_swap_shared_solution(
        &mut self,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    ) -> &mut Self {
        self.arc_swap_shared_solution = Some(arc_swap_shared_solution);
        self
    }
    pub fn loaded_shared_solution(
        &mut self,
        loaded_shared_solution: Guard<Arc<SharedSolution>>,
    ) -> &mut Self {
        self.loaded_shared_solution = Some(loaded_shared_solution);
        self
    }
}

// `new` should be replaced by a builder.
pub trait AlgorithmUtils {
    type Parameters: Parameters;
    type ObjectiveValue;
    type Sol: Solution<ObjectiveValue = Self::ObjectiveValue> + Debug + Clone;
    type I: Default;

    fn builder() -> AlgorithmBuilder<Self::Sol, Self::Parameters, Self::I>;

    fn load_shared_solution(&mut self);

    fn clone_algorithm_solution(&self) -> Self::Sol;

    fn swap_solution(&mut self, solution: Self::Sol);

    // WARN
    // You may have to reintroduce this.
    // fn update_objective_value(&mut self, objective_value: Self::ObjectiveValue);
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

#[derive(PartialEq, Eq, Debug, Clone)]
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

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_start_event));

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_end_event));

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
impl<ActorRequest, ResponseMessage, S, P, I> Agent<ActorRequest, ResponseMessage, S, P, I>
where
    Self: MessageHandler<Req = ActorRequest, Res = ResponseMessage>,
    Algorithm<S, P, I>: ActorBasedLargeNeighborhoodSearch,
    ResponseMessage: Sync + Send + 'static,
    S: Solution,
    P: Parameters,
{
    pub fn handle(&mut self, agent_message: ActorMessage<ActorRequest>) -> Result<()> {
        match agent_message {
            ActorMessage::State(state_link) => self.handle_state_link(state_link)?,
            ActorMessage::Actor(strategic_request_message) => {
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
pub enum ActorMessage<ActorRequest> {
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
    WorkOrders(ActorSpecific),
    WorkerEnvironment,
    TimeEnvironment,
}

#[derive(Debug, Clone)]
pub enum ActorSpecific {
    Strategic(Vec<WorkOrderNumber>),
}
