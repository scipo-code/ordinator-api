pub mod algorithm;
pub mod traits;

use std::fmt::Debug;
use std::fmt::{self};
use std::panic::Location;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::AlgorithmBuilder;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use colored::Colorize;
use flume::Receiver;
use flume::Sender;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorMessage;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Deserialize;
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
    Self: CommandHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
{
    pub actor_id: Id,
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub algorithm: Algorithm,
    pub receiver_from_orchestrator: Receiver<ActorMessage<ActorRequest>>,
    pub sender_to_orchestrator: Sender<Result<ActorResponse>>,
    pub configurations: Arc<ArcSwap<SystemConfigurations>>,
    pub notify_orchestrator: Arc<dyn OrchestratorNotifier>,
    pub error_channel: Sender<anyhow::Error>,
}

// TODO [ ]
// You should consider making a trait here for the agent. That is the best way
// of coding this. You are getting the hang of this and that is the most
// important thing here
impl<ActorRequest, ActorResponse, Algorithm> Actor<ActorRequest, ActorResponse, Algorithm>
where
    Self: CommandHandler<Req = ActorRequest, Res = ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    // This method sends errors to the Orchestrator, which handles the errors
    // from there.
    pub fn run(&mut self) -> ()
    {
        let mut schedule_iteration = ScheduleIteration::default();

        // I do not understand what I should be doing here? I think that the best
        // approach is to understand this as well as I can.

        if let Err(actor_error) = self.algorithm.schedule().with_context(|| {
            format!(
                "{schedule_iteration:#?}\nActor: {}\nLocation: {}",
                self.actor_id,
                Location::caller(),
            )
        }) {
            self.error_channel
                .send(anyhow!(actor_error))
                .expect("If this happens no amount of error handling will save the program")
        }

        schedule_iteration.increment();

        loop {
            while let Ok(message) = self.receiver_from_orchestrator.try_recv() {
                match self.handle(message) {
                    Ok(_) => (),
                    Err(e) => self.error_channel.send(e).expect(
                        "If this happens no amount of error handling will save the program",
                    ),
                }
            }

            let sleep_duration = match self
                .configurations
                .load()
                .throttling
                .get_throttling(&self.actor_id.0)
            {
                Ok(throttle) => throttle,
                Err(err) => {
                    self.error_channel
                        .send(err)
                        .expect("If error channel is down, everything is down");
                    panic!("{}", &self.actor_id.0)
                }
            };

            std::thread::sleep(std::time::Duration::from_millis(sleep_duration));

            if let Err(actor_error) = self
                .algorithm
                // Ahh the issue is that you cannot put this kind of thing in here. The issue comes
                // from the fact that the. The Actor needs to run this.
                // Should the Option be removed? Yes
                .run_lns_iteration()
                .with_context(|| {
                    format!(
                        "{schedule_iteration:#?}\nActor: {}\nLocation: {}",
                        self.actor_id,
                        Location::caller(),
                    )
                })
            {
                self.error_channel
                    .send(actor_error)
                    .expect("If this happens no amount of error handling will save the program")
            }

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
            error_channel: None,
        }
    }
}

// Is what you are getting from this worth it? I do not really
// think so. You will have to make a new function in the
// other
impl<ActorRequest, ActorResponse, Algorithm> CommandHandler
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

    fn handle_request_message(&mut self, _request_message: Self::Req) -> Result<Self::Res>
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
    notify_orchestrator: Option<Arc<dyn OrchestratorNotifier>>,
    //
    communication_for_orchestrator: Option<Communication<ActorRequest, ActorResponse>>,
    error_channel: Option<Sender<anyhow::Error>>,
}

impl<ActorRequest, ActorResponse, SpecificAlgorithm>
    ActorBuilder<ActorRequest, ActorResponse, SpecificAlgorithm>
where
    Actor<ActorRequest, ActorResponse, SpecificAlgorithm>:
        CommandHandler<Req = ActorRequest, Res = ActorResponse>,
    SpecificAlgorithm: ActorBasedLargeNeighborhoodSearch + Send + 'static,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    pub fn build(self) -> Result<Communication<ActorRequest, ActorResponse>>
    {
        let mut agent = Actor {
            actor_id: self.agent_id.unwrap(),
            scheduling_environment: self.scheduling_environment.unwrap(),
            algorithm: self.algorithm.unwrap(),
            receiver_from_orchestrator: self.receiver_from_orchestrator.unwrap(),
            sender_to_orchestrator: self.sender_to_orchestrator.unwrap(),
            configurations: self.configurations.unwrap(),
            notify_orchestrator: self.notify_orchestrator.unwrap(),
            error_channel: self.error_channel.unwrap(),
        };

        let thread_name = agent.actor_id.to_string();

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

    // QUESTION [ ]
    // Do you actually want the `From` trait bound here?
    //
    // What are the alternative options here? I think that the best
    // thing to do
    // Algorithmh call `builder` itself. You should not have to do much.
    pub fn algorithm<F, S, P, I, Ss>(mut self, configure: F) -> Result<Self>
    where
        SpecificAlgorithm: From<algorithm::Algorithm<S, P, I, Ss>>,
        // I do not think that this should be implemented on the
        // You are over engineering it here but I do not see what
        // other options that we have for making this a success.
        S: Solution<Parameters = P> + Debug + Clone + SwapSolution<Ss>,
        Ss: SystemSolutions,
        P: Parameters,
        I: Default,
        F: FnOnce(AlgorithmBuilder<S, P, I, Ss>) -> Result<AlgorithmBuilder<S, P, I, Ss>>,
    {
        let algorithm_builder = algorithm::Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder)?;

        self.algorithm = Some(SpecificAlgorithm::from(algorithm_builder.build()?));

        Ok(self)
    }

    // What is the error here? I think that it has to do with the
    // bounded channel.
    pub fn communication(mut self, error_channel: Sender<anyhow::Error>) -> Self
    {
        let (sender_to_actor, receiver_from_orchestrator): (
            flume::Sender<ActorMessage<ActorRequest>>,
            flume::Receiver<ActorMessage<ActorRequest>>,
        ) = flume::unbounded();

        let (sender_to_orchestrator, receiver_from_actor): (
            flume::Sender<Result<ActorResponse>>,
            flume::Receiver<Result<ActorResponse>>,
        ) = flume::unbounded();

        self.communication_for_orchestrator =
            Some(Communication::new(sender_to_actor, receiver_from_actor));

        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self.error_channel = Some(error_channel);
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

    pub fn notify_orchestrator(mut self, notify_orchestrator: Arc<dyn OrchestratorNotifier>)
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

            write!(f, "{string}")
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
            ConstraintState::Infeasible(reason) => write!(f, "{reason}"),
            ConstraintState::Undetermined => write!(f, "Constraint is not determined yet"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RequestMessage<S, Sc, R, T, C>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
    SchedulingEnvironment(C),
    Update,
}

// You need type safety here I do not see another way around it
//
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponseMessage<S, Sc, R, T, C>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
    SchedulingEnvironment(C),
    Update,
    Succes,
}
