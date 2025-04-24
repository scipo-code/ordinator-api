use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use ordinator_orchestrator_actor_traits::SharedSolution;
// These are mostly build dependencies and should therefore not be found inside of the
// `orchestrator`. The question then becomes what we should do about the building of the
// `actors`
//
// I think that you should go to the gym now. There is an issue here in that
// I do not know what the best way to proceed is for the different.
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;

// There is no reason to update this. I think that the best appraoch is to make the code
// function with the
#[derive(Debug, Clone)]
pub struct AgentFactory {
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
}

// TODO [ ]
// Move every single function into the Agents themselves and have the orchestrator input
// them. This means that almost everything should be made as "non-pub". This is the crucial
// lesson learned from 100s of failures.
// You should test a single instance of the factory. That is the most crucial aspect. Make a single
// function.
impl AgentFactory {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        AgentFactory {
            scheduling_environment,
        }
    }

    // This should be moved to the `Orchestrator`
    pub fn create_shared_solution_arc_swap() -> Arc<ArcSwap<SharedSolution>> {
        let shared_solution_arc_swap = SharedSolution::default();

        Arc::new(ArcSwap::from(Arc::new(shared_solution_arc_swap)))
    }
}
