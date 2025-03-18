use anyhow::Result;

use flume::{Receiver, Sender};

use ordinator_scheduling_environment::{Asset, work_order::WorkOrderNumber};

pub trait OrchestratorNotifier {
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

pub struct ArcSwapSharedSolution(pub ArcSwap<SharedSolution>);

// TODO [ ]
// Rename this crate
// TODO [x]
// Move this to the `core-traits`
// TODO [x]
// Turn the `<Actor>Solution into S: Solution + StrateticInterface`
// because the orchestrator and the other actors should not see the
// concrete implementations of the code
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SharedSolution<Strategic, Tactical, Supervisor, Operational>
where
    Strategic: Solution + StrategicInterface,
    Tactical: Solution + TacticalInterface,
    Supervisor: Solution + SupervisorInterface,
    Operational: Solution + OperationalInterface,
{
    pub strategic: Strategic,
    pub tactical: Tactical,
    pub supervisor: Supervisor,
    pub operational: HashMap<Id, Operational>,
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

pub trait StrategicInterface {}
pub trait TacticalInterface {}
pub trait SupervisorInterface {}
pub trait OperationalInterface {}
