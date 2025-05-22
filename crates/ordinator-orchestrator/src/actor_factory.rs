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
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_strategic_actor::StrategicApi;
use ordinator_strategic_actor::algorithm::strategic_solution::StrategicSolution;
use ordinator_supervisor_actor::SupervisorApi;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
use ordinator_tactical_actor::TacticalApi;
use ordinator_tactical_actor::algorithm::tactical_solution::TacticalSolution;

use crate::NotifyOrchestrator;
use crate::Orchestrator;

// This is not a good practice. You know that you will end up here again at some
// point
//
pub type TotalSystemSolution =
    SystemSolution<StrategicSolution, TacticalSolution, SupervisorSolution, OperationalSolution>;

type ActorFactoryDependencies<Ss> = (
    Arc<Mutex<SchedulingEnvironment>>,
    Arc<ArcSwap<Ss>>,
    Arc<dyn OrchestratorNotifier>,
    Arc<ArcSwap<SystemConfigurations>>,
);

impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        > + Send
        + Sync
        + 'static,
{
    // This is a helper function. This is where the problem becomes appearant
    // It should be removed from the function.
    // TODO [ ] You should move the actor registry out of the factory_dependencies
    // again.
    pub fn extract_factory_dependencies(
        &self,
        asset: &Asset,
    ) -> Result<ActorFactoryDependencies<Ss>>
    {
        Ok((
            Arc::clone(&self.scheduling_environment),
            // This is the issue. FIX Remove this to proceed. Where should it go? I think that the
            // best approach here is the make the code work well with the
            Arc::clone(
                self.system_solutions
                    .lock()
                    .unwrap()
                    .get(asset)
                    .with_context(|| format!("Missing SystemSolution for Asset {asset}"))?,
            ),
            Arc::new(NotifyOrchestrator(
                self.actor_notify
                    .as_ref()
                    .unwrap()
                    .clone()
                    .upgrade()
                    .unwrap(),
            )),
            Arc::clone(&self.system_configurations),
        ))
    }

    // So the `Id` is actually not only an ID, it specifies everything that is
    // unique to that specific actor. I think that is the reason that the system
    // works so well here.
    pub fn start_strategic_actor(&mut self, id: &Id) -> Result<()>
    {
        // Insert an entry on the SchedulingEnvironment
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // Where should the code for the id come from? You need to make sure that you
        // understand the process, correctly.
        //
        // Where should the id come from? I think that the best place to retrieve them
        // from is the data itself. Usually this comes from either the database
        // or from the API endpoint. What does that mean for the remaining part
        // of the system. You should add something from the
        //
        //
        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <StrategicApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("Could not create StrategicActor for Asset {}", id.asset()))?;

        // You should make a method for
        // `actor_registries.lock().unwrap().get_mut(id.asset()).expect()`
        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .strategic_agent_sender = communication;
        Ok(())
    }

    pub fn start_tactical_actor(&mut self, id: &Id) -> Result<()>
    {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <TacticalApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("Could not create TacticalActor for Asset {}", id.asset()))?;

        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .tactical_agent_sender = communication;
        Ok(())
    }

    // TODO [ ] - Move the ActorSpecification into the SchedulingEnvironment.
    pub fn start_supervisor_actor(&mut self, id: &Id) -> Result<()>
    {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        let communication = <SupervisorApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("Could not create supervisorActor for Asset {}", id.asset()))?;

        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .supervisor_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }

    // You should only ever build the actor based on the state that is present in
    // the `SchedulingEnvironment` This actor is different. We have to insert a
    // different component into the system here. The best approach would
    // probably be to
    pub fn start_operational_actor(&mut self, id: &Id) -> Result<()>
    {
        // TODO [ ] - Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO [ ] - Determine what to do about the `ID` here.
        let communication = <OperationalApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            build_dependencies.3,
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("Could not create OperationalActor for Asset {}", id.asset()))?;

        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .operational_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }
}
