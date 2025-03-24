pub mod delegate;
pub mod marginal_fitness;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Result;
use delegate::Delegate;
use flume::Receiver;
use flume::Sender;
use marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;

pub trait OrchestratorNotifier: Send + Sync + 'static
{
    fn notify_all_agents_of_work_order_change(
        &self,
        work_orders: Vec<WorkOrderNumber>,
        asset: &Asset,
    ) -> Result<()>;
}

pub struct Communication<Req, Res>
{
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

pub trait SharedSolutionTrait: Clone
{
    type Strategic: StrategicInterface;
    type Tactical: TacticalInterface;
    type Supervisor: SupervisorInterface;
    type Operational: OperationalInterface;

    fn strategic(&self) -> &Self::Strategic;
    fn tactical(&self) -> &Self::Tactical;
    fn supervisor(&self) -> &Self::Supervisor;
    fn operational(&self, id: &Id) -> &Self::Operational;
}

// You are out in the woods here. You should keep up the work and focus on
// making the You are not making this in the correct way. I think that a better
// approach is to
//
// TODO [ ]
// Make this work with the correct way of designing
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

    fn strategic(&self) -> &Self::Strategic
    {
        &self.strategic
    }

    fn tactical(&self) -> &Self::Tactical
    {
        &self.tactical
    }

    fn supervisor(&self) -> &Self::Supervisor
    {
        &self.supervisor
    }

    fn operational(&self, id: &Id) -> &Self::Operational
    {
        self.operational
            .get(id)
            .expect("querieed nonexisting operaional agent")
    }

    // You could implement the pointer swapping here. Hmm... that might not be the
    // best idea.
}

pub trait Parameters
where
    Self: Sized,
{
    type Key;
    type Options;

    fn from_source(
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

pub trait Solution
{
    type ObjectiveValue;
    type Parameters;

    // QUESTION
    // Is this a good idea to create the Solution? I actually believe that it
    // is!
    fn new(parameters: &Self::Parameters) -> Self;

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue);
}

pub trait MessageHandler
{
    type Req;
    type Res;

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

//
// There should only be a single interface here there should be a
// a set of standard operations that every solution should inplement
// this is to make sure that you do not make stray impl blocks and
// TODO [ ]
// Make a solution interface that is common
pub trait StrategicInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn scheduled_task(&self, work_order_number: &WorkOrderNumber) -> Option<Option<Period>>;
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
    fn marginal_fitness_for_operational_actor<'a>(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Vec<&'a MarginalFitness>;
}

#[derive(Clone)]
pub enum ActorMessage<ActorRequest>
{
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
