use anyhow::Result;

use flume::{Receiver, Sender};

use ordinator_scheduling_environment::{work_order::WorkOrderNumber, Asset};

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
