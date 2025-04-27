use std::sync::Arc;

use arc_swap::ArcSwap;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::SharedSolution;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
// These are mostly build dependencies and should therefore not be found inside of the
// `orchestrator`. The question then becomes what we should do about the building of the
// `actors`
//
// I think that you should go to the gym now. There is an issue here in that
// I do not know what the best way to proceed is for the different.
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_strategic_actor::StrategicApi;

use crate::Orchestrator;

// There is no reason to update this. I think that the best appraoch is to make the code
// function with the. You need methods for starting the start manually and from the
// SchedulingEnvironment. You have had this dilemma many times before. You cannot simply
// shourcurcuit the system in this case.
// Maybe having the trait is not a bad idea here.
// There is something here that you have not thought about. You need to create the system
// so that each of the different parts of the system uses the correct dataflow for making
// this work. The code should be created so that the Actors knows how to
// What should happen if the code does two different calls to start strategic? I think that
// the best approach in that regard will. Ideally you should set the value of the `SharedSolution`
// in here as well. I think that the best approach is to make the system work with the
// Option<Solution>.
// SharedSolution, has to be an option. There is no other way about it.
// OBSTACLE
// You have felt for a long time that you want to disregard the `Option`s as much as
// possible that means that this is the way. You might not always actually want to
// have a strategic and tactical agent, and you will have to choose the thing that you
// want the most in each deployment. And then we can also load from the Database itself.
//
// Take a break, and what should you then do after the break? So an API call should generate
// an entry in the SchedulingEnvironment. Everything that has to do with the Database should
// be done as a cross cutting concern. 
impl<Ss> Orchestrator<Ss> {
    pub fn extract_factory_dependencies(&self) -> () {}
    pub fn start_strategic_actor(&mut self) {
        // Insert an entry on the SchedulingEnvironment
        //
        //

        // Where should the code for the id come from? You need to make sure that you understand the
        // process, correctly.
        let communication = <StrategicApi as ActorFactory>::construct_actor(
            id,
            scheduling_environment_guard,
            shared_solution_arc_swap,
            notify_orchestrator,
            system_configurations,
        );

        strategic_factory(
            id,
            scheduling_environment_guard,
            shared_solution_arc_swap,
            notify_orchestrator,
            system_configurations,
        );

        self.agent_registry.
    }

    pub fn start_supervisor_actor(&mut self) {
        let supervisor_agent_addr = supervisor_factory(
            id,
            scheduling_environment_guard,
            shared_solution_arc_swap,
            notify_orchestrator,
            system_configurations,
        );
        self.agent_registries
            .get_mut(&asset)
            .unwrap()
            .add_supervisor_agent(
                id_string.clone(),
                supervisor_agent_addr.expect("Could not create SupervisorAgent"),
            );
    }
}

pub fn create_shared_solution_arc_swap<Ss>() -> Arc<ArcSwap<Ss>>
where
    Ss: SharedSolutionTrait,
{
    let shared_solution_arc_swap = SharedSolution::default();

    Arc::new(ArcSwap::from(Arc::new(shared_solution_arc_swap)))
}
