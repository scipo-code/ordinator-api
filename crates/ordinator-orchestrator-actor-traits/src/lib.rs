pub mod delegate;
pub mod marginal_fitness;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use chrono::DateTime;
use chrono::Utc;
use delegate::Delegate;
use flume::Receiver;
use flume::Sender;
use marginal_fitness::MarginalFitness;
use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;

pub trait OrchestratorNotifier: Send + Sync + 'static
{
    fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()>;
}

pub struct Communication<RequestMessage, Res>
{
    sender: Sender<ActorMessage<RequestMessage>>,
    pub receiver: Receiver<Result<Res>>,
}

// StateLink is not a request. It is something different
// Ahh this is good every Request message from each of the actors
// should implement a `RequestMessage`. It is a little weird to
// reuse the `Req` like this. You need to remember this to see
// what you will learn from it.
//
// You are misunderstanding this because you are not using the
// generics in the correct way. There is something to learn here.
impl<RequestMessage, Res> Communication<RequestMessage, Res>
{
    pub fn new(
        sender: Sender<ActorMessage<RequestMessage>>,
        receiver: Receiver<Result<Res>>,
    ) -> Self
    {
        Self { sender, receiver }
    }

    // This is being wrapped twice. I think that the best approach is to
    // make the system function with.
    pub fn from_agent(&self, message: RequestMessage) -> Result<()>
    {
        // What is it that you need to do here? You should
        let message = ActorMessage::Actor(message);
        self.sender.send(message).map_err(|e| anyhow!(e.to_string() )).context("The Actor has stopped running. If the reason for this is not obvious, it means that the error handling should be extended.")
    }

    pub fn from_actor(&self) -> Res
    {
        self.receiver.recv().unwrap().unwrap()
    }

    pub fn from_orchestrator(&self, state_link: StateLink)
    {
        let message = ActorMessage::State(state_link);
        self.sender.send(message).expect("The Actor has stopped running. If the reason for this is not obvious, it means that the error handling should be extended.");
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SystemSolution<S, T, U, V>
where
    S: StrategicInterface,
    T: TacticalInterface,
    U: SupervisorInterface,
    // FIX [ ]
    // This `Solution` should be removed.
    V: OperationalInterface + Solution,
{
    pub strategic: Option<S>,
    pub tactical: Option<T>,
    pub supervisor: Option<U>,
    pub operational: HashMap<Id, V>,
}

// This is made completely wrong. I am not sure what the
// best approach of solving it will be.
pub trait SystemSolutionTrait: Clone
{
    type Strategic: StrategicInterface;
    type Tactical: TacticalInterface;
    type Supervisor: SupervisorInterface;
    type Operational: OperationalInterface + Solution;

    fn new() -> Self;
    fn strategic(&self) -> &Self::Strategic;

    fn strategic_swap(&mut self, id: &Id, solution: Self::Strategic)
    where
        Self::Strategic: Solution;
    fn tactical(&self) -> &Self::Tactical;

    fn tactical_swap(&mut self, id: &Id, solution: Self::Tactical)
    where
        Self::Tactical: Solution;
    fn supervisor(&self) -> &Self::Supervisor;

    fn supervisor_swap(&mut self, id: &Id, solution: Self::Supervisor)
    where
        Self::Supervisor: Solution;
    fn operational(&self, id: &Id) -> &Self::Operational;

    fn all_operational(&self) -> HashSet<Id>;
    // If you make all Id's internal you could simply work on those?
    fn operational_swap(&mut self, id: &Id, solution: Self::Operational)
    where
        Self::Operational: Solution;
}

// You are out in the woods here. You should keep up the work and focus on
// making the You are not making this in the correct way. I think that a better
// approach is to
//
// TODO [ ]
// Make this work with the correct way of designing
#[allow(dead_code, unused_variables)]
impl<S, T, U, V> SystemSolutionTrait for SystemSolution<S, T, U, V>
where
    S: StrategicInterface,
    T: TacticalInterface,
    U: SupervisorInterface,
    V: OperationalInterface + Solution,
{
    type Operational = V;
    type Strategic = S;
    type Supervisor = U;
    type Tactical = T;

    fn new() -> Self
    {
        Self {
            strategic: None,
            tactical: None,
            supervisor: None,
            operational: HashMap::default(),
        }
    }

    fn strategic(&self) -> &Self::Strategic
    {
        self.strategic.as_ref().unwrap()
    }

    fn tactical(&self) -> &Self::Tactical
    {
        self.tactical.as_ref().unwrap()
    }

    fn supervisor(&self) -> &Self::Supervisor
    {
        self.supervisor.as_ref().unwrap()
    }

    fn operational(&self, id: &Id) -> &Self::Operational
    {
        self.operational
            .get(id)
            .expect("querieed nonexisting operaional agent")
    }

    // Can you even do this? Is this allowed? I do not t
    fn operational_swap(&mut self, id: &Id, solution: Self::Operational)
    where
        Self::Operational: Solution,
    {
        self.operational.insert(id.clone(), solution);
    }

    fn strategic_swap(&mut self, id: &Id, solution: Self::Strategic)
    where
        Self::Strategic: Solution,
    {
        self.strategic = Some(solution);
    }

    fn tactical_swap(&mut self, id: &Id, solution: Self::Tactical)
    where
        Self::Tactical: Solution,
    {
        todo!()
    }

    fn supervisor_swap(&mut self, id: &Id, solution: Self::Supervisor)
    where
        Self::Supervisor: Solution,
    {
        todo!()
    }

    fn all_operational(&self) -> HashSet<Id>
    {
        self.operational.keys().cloned().collect()
    }

    // You could implement the pointer swapping here. Hmm... that might not be the
    // best idea.
}

pub trait Parameters
where
    Self: Sized,
{
    type Key;

    /// Who should build the parameters. That is the key question here.
    /// Do you want to mutate it?
    ///
    /// I really do not like this trait declaration. Something has to change?
    fn from_source(
        id: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>;

    /// WARNING
    /// This method can become extremely complex in a practical setting.
    /// You should do.
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    );

    // TODO [ ]
    // Add methods for updating configurations.
}

// There is something that I do not like about having `new` here
// I think that the best option is to make the system work with the
// `from` trait. Meaning that we should focus on making the system
// work with the
// Should this function have an option or not? Yes it should.
pub trait Solution
{
    type ObjectiveValue;
    type Parameters;

    // The weightings are found inside of the
    // `Solution`
    // QUESTION
    // Is this a good idea to create the Solution? I actually believe that it
    // is!
    // Should you have the options here? I think that you should derive the...
    //
    // The solution should only contain the things that actually change.
    fn new(parameters: &Self::Parameters) -> Self;

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue);
}

pub trait MessageHandler
{
    type Req;
    type Res;

    // This has the wrong kind of name. I do not see what else I could do here.
    // Maybe I should strive
    // Here it wraps the `Req` in the `ActorMessage` I do not think that this
    // is the best way of doing it
    fn handle(&mut self, actor_message: ActorMessage<Self::Req>) -> Result<Self::Res>
    {
        match actor_message {
            ActorMessage::State(state_link) => self.handle_state_link(state_link),
            ActorMessage::Actor(actor_request) => self.handle_request_message(actor_request),
        }
    }

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Self::Res>;

    fn handle_request_message(&mut self, request_message: Self::Req) -> Result<Self::Res>;
}

// There should only be a single interface here there should be a
// a set of standard operations that every solution should inplement
// this is to make sure that you do not make stray impl blocks and
// TODO [ ]
// Make a solution interface that is common
pub trait StrategicInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn scheduled_task(&self, work_order_number: &WorkOrderNumber) -> Option<&Option<Period>>;

    fn supervisor_tasks(&self, periods: &[Period]) -> HashMap<WorkOrderNumber, Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, Period>;
}

pub trait TacticalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&DateTime<Utc>, &DateTime<Utc>)>;

    fn tactical_period(&self, work_order_number: &WorkOrderNumber) -> Option<&Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, BTreeMap<ActivityNumber, Day>>;
}

// This is a core type that each `Actor` should implement, I think
// that it should be part of a trait but which is a little difficult
// to tell.
// QUESTION
// What is this type set in the world to do?
// The goal of it is to make sure that the `Actor` can make
// custom logic internally depending on where they know the
// work order to be located. This is crucial to respect
// business logic.
#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub enum WhereIsWorkOrder<T>
{
    Strategic,
    Tactical(T),
    #[default]
    NotScheduled,
}
impl<T> WhereIsWorkOrder<T>
{
    pub fn is_tactical(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Tactical(_))
    }
}

pub trait SupervisorInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn delegated_tasks(&self, operational_agent: &Id) -> HashSet<WorkOrderActivity>;
    // Where should the `Delegate` be located?
    // I believe that the best place is in the `actor-core` the issue is that you
    // wanted to use these interface for the `orchestrator` as well and that is not
    // what you actually want. You need the orchestrator to have the delegate as
    // well in general you would like to have the `Solutions` able to be
    // exported directly from the `orchestrator`.
    // QUESTION
    // TODO [ ]
    // Should you simply move the module into this crate? Yes I think that is a good
    // idea.
    // What should the `delegates` here be called? Remember that you can use
    // associated types to fix this in the correct way. The best approach would
    // QUESTION
    // What exactly is the function doing? It is for a specific actor finding every
    // work order activity that is relevant for this actor. The function is
    // basically returning a `SortedSolution` for the `SupervisorSolution`.
    //
    // I think...
    // The best approach here is to make something that we make a trait for each and
    // then afterwards you look at all four of these traits and then make a
    // common! Bullseye! This is the approach abstract should always be created
    // with evidence not blind faith.
    fn delegates_for_agent(&self, operational_agent: &Id) -> HashMap<WorkOrderActivity, Delegate>;
    fn count_delegate_types(&self, operational_agent: &Id) -> (u64, u64, u64);
}
// The `solution` should be updated on the `SharedSolution` not the
// individual solution. These interfaces are implemented on the
// individual solution.
pub trait OperationalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    // This function is completely on the wrong level of abstraction, this
    // feeling is what should guide you towards correct behavior.
    // This interface should be implemented by the `operational` actor.
    // And this means that the most important thing is the that the
    // method cannot see the solution from the `supervisor` are you missing
    // something here?
    //
    // This should not be a `Vec` correct? A `WorkOrderActivity` is unique to
    // this actor? Yes
    fn marginal_fitness_for_operational_actor<'a>(
        &'a self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<&'a MarginalFitness>;
}

// You should make an API on the `Communication` struct. What other approach
// should I take.
#[derive(Clone)]
pub enum ActorMessage<ActorRequest>
{
    State(StateLink),
    Actor(ActorRequest),
    // Yes so options should be included here as part of what needs to be created for
    // this to work. I believe that the best approach here will be to make something
    // that
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
pub enum StateLink
{
    WorkOrders(ActorSpecific),
    WorkerEnvironment,
    TimeEnvironment,
}

#[derive(Debug, Clone)]
pub enum ActorSpecific
{
    Strategic(Vec<WorkOrderNumber>),
}

pub trait ActorFactory<Ss>
where
    Ss: SystemSolutionTrait + Sync + Send,
{
    type Communication;

    fn construct_actor(
        id: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        system_solution_arc_swap: Arc<ArcSwap<Ss>>,
        notify_orchestrator: Arc<dyn OrchestratorNotifier>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    ) -> Result<Self::Communication>;
}
