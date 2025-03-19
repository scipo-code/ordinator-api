use std::collections::HashMap;
use std::sync::MutexGuard;

use anyhow::Result;
use flume::Receiver;
use flume::Sender;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;

pub trait OrchestratorNotifier: Send + Sync + 'static {
    fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()>;
}

pub struct Communication<Req, Res> {
    pub sender: Sender<Req>,
    pub receiver: Receiver<Result<Res>>,
}

// TODO [ ]
// Rename this crate
// TODO [x]
// Move this to the `core-traits`
// TODO [x]
// Turn the `<Actor>Solution into S: Solution + StrateticInterface`
// because the orchestrator and the other actors should not see the
// concrete implementations of the code
// I think that the best approach here is to make this
// Should the code implement this? I am not really sure here
// I think that
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SharedSolution<S, T, U, V>
where
    S: StrategicInterface,
    T: TacticalInterface,
    U: SupervisorInterface,
    V: OperationalInterface,
{
    pub strategic: S,
    pub tactical: T,
    pub supervisor: U,
    pub operational: HashMap<Id, V>,
}

pub trait SharedSolutionTrait {
    type Strategic;
    type Tactical;
    type Supervisor;
    type Operational;
}

impl<S, T, U, V> SharedSolutionTrait for SharedSolution<S, T, U, V>
where
    S: StrategicInterface,
    T: TacticalInterface,
    U: SupervisorInterface,
    V: OperationalInterface,
{
    type Operational = V;
    type Strategic = S;
    type Supervisor = U;
    type Tactical = T;

    // You could implement the pointer swapping here. Hmm... that might not be the
    // best idea.
}

pub trait Parameters
where
    Self: Sized,
{
    type Key;
    type Options;

    fn new(
        id: &Id,
        options: &Self::Options,
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

pub trait Solution {
    type ObjectiveValue;
    type Parameters;

    // QUESTION
    // Is this a good idea to create the Solution? I actually believe that it
    // is!
    fn new(parameters: &Self::Parameters) -> Self;

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue);
}

pub trait MessageHandler {
    type Req;
    type Res;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()>;

    fn handle_request_message(&mut self, request_message: Self::Req) -> Result<Self::Res>;
}

pub trait StrategicInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
}
pub trait TacticalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
}
pub trait SupervisorInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
}
pub trait OperationalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
}

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
