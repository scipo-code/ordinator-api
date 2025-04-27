use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_configuration::SystemConfigurations;
use ordinator_operational_actor::OperationalApi;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::OrchestratorNotifier;
use ordinator_orchestrator_actor_traits::SystemSolution;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::resources::Id;
// These are mostly build dependencies and should therefore not be found inside of the
// `orchestrator`. The question then becomes what we should do about the building of the
// `actors`
//
// I think that you should go to the gym now. There is an issue here in that
// I do not know what the best way to proceed is for the different.
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_strategic_actor::StrategicApi;
use ordinator_strategic_actor::algorithm::strategic_solution::StrategicSolution;
use ordinator_supervisor_actor::SupervisorApi;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
use ordinator_tactical_actor::TacticalApi;
use ordinator_tactical_actor::algorithm::tactical_solution::TacticalSolution;

use crate::NotifyOrchestrator;
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
//
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutionTrait<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        > + Send
        + Sync
        + 'static,
{
    pub fn extract_factory_dependencies(
        &self,
        asset: &Asset,
    ) -> Result<(
        Arc<Mutex<SchedulingEnvironment>>,
        Arc<ArcSwap<Ss>>,
        Box<dyn OrchestratorNotifier>,
        Arc<ArcSwap<SystemConfigurations>>,
    )> {
        Ok((
            Arc::clone(&self.scheduling_environment),
            Arc::clone(
                self.system_solutions
                    .get(asset)
                    .context("Asset not available in for the SystemSolution")?,
            ),
            Box::new(NotifyOrchestrator(
                self.actor_notify
                    .as_ref()
                    .unwrap()
                    .clone()
                    .upgrade()
                    .unwrap(),
            )),
            Arc::clone(
                self.system_configurations
                    .get(asset)
                    .context("SystemConfigurations not available for the Asset")?,
            ),
        ))
    }

    // So the `Id` is actually not only an ID, it specifies everything that is unique to that specific
    // actor. I think that is the reason that the system works so well here.
    pub fn start_strategic_actor(&mut self, id: &Id) -> Result<()> {
        // Insert an entry on the SchedulingEnvironment
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // Where should the code for the id come from? You need to make sure that you understand the
        // process, correctly.
        //
        // Where should the id come from? I think that the best place to retrieve them from is the
        // data itself. Usually this comes from either the database or from the API endpoint. What
        // does that mean for the remaining part of the system. You should add something from the
        //
        //
        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <StrategicApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
        )
        .with_context(|| format!("Could not create StrategicActor for Asset {}", id.asset()))?;

        self.agent_registries
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .strategic_agent_sender = communication;
        Ok(())
    }
    pub fn start_tactical_actor(&mut self, id: &Id) -> Result<()> {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <TacticalApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
        )
        .with_context(|| format!("Could not create TacticalActor for Asset {}", id.asset()))?;

        self.agent_registries
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .tactical_agent_sender = communication;
        Ok(())
    }

    // TODO [ ] - Move the ActorSpecification into the SchedulingEnvironment.
    pub fn start_supervisor_actor(&mut self, id: &Id) -> Result<()> {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        let communication = <SupervisorApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
        )
        .with_context(|| format!("Could not create supervisorActor for Asset {}", id.asset()))?;

        self.agent_registries
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .supervisor_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }

    // You should only ever build the actor based on the state that is present in the `SchedulingEnvironment`
    // This actor is different. We have to insert a different component into the system here.
    // The best approach would probably be to
    pub fn start_operational_actor(&mut self, id: &Id) -> Result<()> {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <OperationalApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
        )
        .with_context(|| format!("Could not create OperationalActor for Asset {}", id.asset()))?;

        self.agent_registries
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .operational_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }
}

// QUESTION [ ] How to create a
pub fn create_shared_solution_arc_swap<Ss>() -> Arc<ArcSwap<Ss>>
where
    Ss: SystemSolutionTrait,
{
    let shared_solution_arc_swap = SystemSolution::new();

    Arc::new(ArcSwap::from(Arc::new(shared_solution_arc_swap)))
}
