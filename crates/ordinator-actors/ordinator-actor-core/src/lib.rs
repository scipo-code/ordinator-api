pub mod algorithm;
pub mod delegate;
pub mod traits;

use std::fmt::Debug;
use std::fmt::{self};
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::AlgorithmBuilder;
use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use colored::Colorize;
use flume::Receiver;
use flume::Sender;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

use self::traits::ActorBasedLargeNeighborhoodSearch;

// I do not know if there is
// TODO [ ] FIX [ ]
// You should reuse the trait bounds on the Agent and the Algorithm.
pub struct Actor<ActorRequest, ActorResponse, Algorithm>
where
    // What should you do here with the
    // You should implement the MessageHandler for all of the
    // Actors this means that you need to create a blanket
    // implementation, and then the actors will have to supply
    // an implementation of the functions needed to actually
    // perform the required operations.
    // TODO [ ]
    // Look into whether it is possible for you to make a
    // blanket implementation that simply makes the
    // Actor implementations provide functions.
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    pub agent_id: Id,
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm,
    pub receiver_from_orchestrator: Receiver<ActorMessage<ActorRequest>>,
    pub sender_to_orchestrator: Sender<Result<ActorResponse>>,
    pub configurations: Arc<ArcSwap<SystemConfigurations>>,
    pub notify_orchestrator: Box<dyn OrchestratorNotifier>,
}

// TODO [ ]
// You should consider making a trait here for the agent. That is the best way
// of coding this. You are getting the hang of this and that is the most
// important thing here
impl<ActorRequest, ActorResponse, Algorithm> Actor<ActorRequest, ActorResponse, Algorithm>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    pub fn run(&mut self) -> Result<()>
    {
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

            // There is no good way of doing this. You simply have to make the
            // system so that it will always return to the correct state.
            // You have to circumvent the `configurations` here and simply
            // have the system run on the data. I do not see what other options
            // that we have here. You could simply use an arc swap again. I think
            // that is a good decision. You should use `ArcSwap` for the
            // configurations as well. Yes or no? Yes, that is a good idea.
            //
            // I think that this is the best way of doing it, but how should we
            // get the code inside of the algorithm? The idea of putting it into the
            // algorithm is probably not such a good idea. I cannot see what other
            // appraoch that we should.
            // So is this even possible to do here? I am not really sure. I think that the
            // best approach is to make the system. How do I make a function here that
            // accepts the correct number of.
            // You have to make a method for getting the functionality out.

            self.algorithm
                .run_lns_iteration(self.configurations.load(), &self.agent_id)
                .with_context(|| format!("{:#?}", schedule_iteration))
                .unwrap();

            schedule_iteration.increment();
        }
    }

    pub fn builder() -> ActorBuilder<ActorRequest, ActorResponse, Algorithm>
    {
        ActorBuilder {
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

// Is what you are getting from this worth it? I do not really
// think so. You will have to make a new function in the
// other
impl<ActorRequest, ActorResponse, Algorithm> MessageHandler
    for Actor<ActorRequest, ActorResponse, Algorithm>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    type Req = ActorRequest;
    type Res = ActorResponse;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Self::Res>
    {
        match state_link {
            StateLink::WorkOrders(_actor_specific) => todo!(),
            StateLink::WorkerEnvironment => todo!(),
            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(&mut self, request_message: Self::Req) -> Result<Self::Res>
    {
        // The individual actor has to implement this
        todo!();
    }
}

pub struct ActorBuilder<ActorRequest, ActorResponse, Algorithm>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    agent_id: Option<Id>,
    scheduling_environment: Option<Arc<Mutex<SchedulingEnvironment>>>,
    algorithm: Option<Algorithm>,
    receiver_from_orchestrator: Option<Receiver<ActorMessage<ActorRequest>>>,
    sender_to_orchestrator: Option<Sender<Result<ActorResponse>>>,
    configurations: Option<Arc<ArcSwap<SystemConfigurations>>>,
    notify_orchestrator: Option<Box<dyn OrchestratorNotifier>>,
    //
    communication_for_orchestrator:
        Option<Communication<ActorMessage<ActorRequest>, ActorResponse>>,
}

impl<ActorRequest, ActorResponse, Algorithm> ActorBuilder<ActorRequest, ActorResponse, Algorithm>
where
    Actor<ActorRequest, ActorResponse, Algorithm>:
        MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch + Send + 'static,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    pub fn build(self) -> Result<Communication<ActorMessage<ActorRequest>, ActorResponse>>
    {
        let mut agent = Actor {
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

        Ok(self.communication_for_orchestrator.unwrap())
    }

    pub fn agent_id(mut self, agent_id: Id) -> Self
    {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn scheduling_environment(
        mut self,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self
    {
        self.scheduling_environment = Some(scheduling_environment);
        self
    }

    pub fn algorithm<F, S, P, I, Ss>(mut self, configure: F) -> Self
    where
        Algorithm: From<algorithm::Algorithm<S, P, I, Ss>>,
        S: Solution<Parameters = P> + Debug + Clone,
        Ss: SharedSolutionTrait,
        P: Parameters,
        I: Default,
        F: FnOnce(AlgorithmBuilder<S, P, I, Ss>) -> AlgorithmBuilder<S, P, I, Ss>,
    {
        let algorithm_builder = algorithm::Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder);

        self.algorithm = Some(Algorithm::from(algorithm_builder.build()));
        self
    }

    pub fn communication(mut self) -> Self
    {
        let (sender_to_agent, receiver_from_orchestrator): (
            flume::Sender<ActorMessage<ActorRequest>>,
            flume::Receiver<ActorMessage<ActorRequest>>,
        ) = flume::unbounded();

        let (sender_to_orchestrator, receiver_from_agent): (
            flume::Sender<std::result::Result<ActorResponse, anyhow::Error>>,
            flume::Receiver<std::result::Result<ActorResponse, anyhow::Error>>,
        ) = flume::unbounded();

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
    ) -> Self
    {
        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self
    }

    pub fn sender_to_orchestrator(
        mut self,
        sender_to_orchestrator: Sender<Result<ActorResponse>>,
    ) -> Self
    {
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self
    }

    pub fn configurations(mut self, configurations: Arc<ArcSwap<SystemConfigurations>>) -> Self
    {
        self.configurations = Some(configurations);
        self
    }

    pub fn notify_orchestrator(mut self, notify_orchestrator: Box<dyn OrchestratorNotifier>)
    -> Self
    {
        self.notify_orchestrator = Some(notify_orchestrator);
        self
    }
}

#[derive(Default)]
pub struct ScheduleIteration
{
    loop_iteration: u64,
}

impl ScheduleIteration
{
    pub fn increment(&mut self)
    {
        self.loop_iteration += 1;
    }
}

impl fmt::Debug for ScheduleIteration
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
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

/// This type is the primary message type that all agents should receive.
/// All agents should have the `StateLink` and each agent then have its own
/// ActorRequest which is specifically created for each agent.
// THIS should most likely be removed or refactored.
#[derive(Debug, Serialize)]
pub enum AlgorithmState<T>
{
    Feasible,
    Infeasible(T),
}

impl<T> AlgorithmState<T>
{
    pub fn infeasible_cases_mut(&mut self) -> Option<&mut T>
    {
        match self {
            AlgorithmState::Feasible => None,
            AlgorithmState::Infeasible(infeasible_cases) => Some(infeasible_cases),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ConstraintState<Reason>
{
    Feasible,
    Infeasible(Reason),
    Undetermined,
}

impl<Reason> fmt::Display for ConstraintState<Reason>
where
    Reason: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            ConstraintState::Feasible => write!(f, "FEASIBLE"),
            ConstraintState::Infeasible(reason) => write!(f, "{}", reason),
            ConstraintState::Undetermined => write!(f, "Constraint is not determined yet"),
        }
    }
}
