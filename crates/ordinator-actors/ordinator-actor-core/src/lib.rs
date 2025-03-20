pub mod algorithm;
pub mod delegate;
pub mod traits;

use std::fmt::Debug;
use std::fmt::{self};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use algorithm::Algorithm;
use algorithm::AlgorithmBuilder;
use anyhow::Context;
use anyhow::Result;
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
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

use self::traits::ActorBasedLargeNeighborhoodSearch;

// TODO [ ] FIX [ ]
// You should reuse the trait bounds on the Agent and the Algorithm.
pub struct Actor<ActorRequest, ActorResponse, S, P, I, Ss>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I, Ss>: ActorBasedLargeNeighborhoodSearch,
    Ss: SharedSolutionTrait,
    S: Solution,
    P: Parameters,
{
    agent_id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm<S, P, I, Ss>,
    pub receiver_from_orchestrator: Receiver<ActorMessage<ActorRequest>>,
    pub sender_to_orchestrator: Sender<Result<ActorResponse>>,
    pub configurations: Arc<RwLock<SystemConfigurations>>,
    pub notify_orchestrator: Box<dyn OrchestratorNotifier>,
}

// TODO [ ]
// You should consider making a trait here for the agent. That is the best way
// of coding this. You are getting the hang of this and that is the most
// important thing here.
impl<ActorRequest, ActorResponse, S, P, I, Ss> Actor<ActorRequest, ActorResponse, S, P, I, Ss>
where
    Self: MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I, Ss>: ActorBasedLargeNeighborhoodSearch,
    Ss: SharedSolutionTrait,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone,
    P: Parameters,
    I: Default,
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

            self.algorithm
                .run_lns_iteration()
                .with_context(|| format!("{:#?}", schedule_iteration))
                .unwrap();

            schedule_iteration.increment();
        }
    }

    pub fn builder() -> ActorBuilder<ActorRequest, ActorResponse, S, P, I, Ss>
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

pub struct ActorBuilder<ActorRequest, ActorResponse, S, P, I, Ss>
where
    Algorithm<S, P, I, Ss>: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone,
    P: Parameters,
    I: Default,
    Ss: SharedSolutionTrait,
{
    agent_id: Option<Id>,
    scheduling_environment: Option<Arc<Mutex<SchedulingEnvironment>>>,
    algorithm: Option<Algorithm<S, P, I, Ss>>,
    receiver_from_orchestrator: Option<Receiver<ActorMessage<ActorRequest>>>,
    sender_to_orchestrator: Option<Sender<Result<ActorResponse>>>,
    configurations: Option<Arc<RwLock<SystemConfigurations>>>,
    notify_orchestrator: Option<Box<dyn OrchestratorNotifier>>,
    //
    communication_for_orchestrator:
        Option<Communication<ActorMessage<ActorRequest>, ActorResponse>>,
}

impl<ActorRequest, ActorResponse, S, P, I, Ss>
    ActorBuilder<ActorRequest, ActorResponse, S, P, I, Ss>
where
    Actor<ActorRequest, ActorResponse, S, P, I, Ss>:
        MessageHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm<S, P, I, Ss>: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
    S: Solution + Debug + Clone + Send + Sync + 'static,
    P: Parameters + Send + Sync + 'static,
    I: Default + Send + Sync + 'static,
    Ss: SharedSolutionTrait + Send + Sync + 'static,
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

    pub fn algorithm<F>(mut self, configure: F) -> Self
    where
        S: Solution<Parameters = P> + Debug + Clone,
        P: Parameters,
        I: Default,
        F: FnOnce(AlgorithmBuilder<S, P, I, Ss>) -> AlgorithmBuilder<S, P, I, Ss>,
    {
        let algorithm_builder = Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder);

        self.algorithm = Some(algorithm_builder.build());
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

    pub fn configurations(mut self, configurations: Arc<RwLock<SystemConfigurations>>) -> Self
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

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub enum WhereIsWorkOrder<T>
{
    Strategic,
    Tactical(T),
    #[default]
    NotScheduled,
}

// Should the new function take in the `parameters` as an function parameter?
// FIX
// This could be generic! I think that it should.
impl<ActorRequest, ResponseMessage, S, P, I, Ss> Actor<ActorRequest, ResponseMessage, S, P, I, Ss>
where
    Self: MessageHandler<Req = ActorRequest, Res = ResponseMessage>,
    Algorithm<S, P, I, Ss>: ActorBasedLargeNeighborhoodSearch,
    ResponseMessage: Sync + Send + 'static,
    S: Solution,
    P: Parameters,
    Ss: SharedSolutionTrait,
{
    pub fn handle(&mut self, agent_message: ActorMessage<ActorRequest>) -> Result<()>
    {
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
