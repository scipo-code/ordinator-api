use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use arc_swap::Guard;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use super::supervisor_parameters::SupervisorParameters;

pub type SupervisorObjectiveValue = u64;

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SupervisorSolution
{
    pub(crate) objective_value: SupervisorObjectiveValue,
    pub(crate) operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate>,
}

impl Solution for SupervisorSolution
{
    type ObjectiveValue = SupervisorObjectiveValue;
    type Parameters = SupervisorParameters;

    fn new(parameters: &Self::Parameters) -> Self
    {
        // The SupervisorParameters should have knowledge of the agents.

        let operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate> = parameters
            .supervisor_work_orders
            .iter()
            .flat_map(|(won, inner)| {
                inner.iter().flat_map(|(acn, sp)| {
                    // So here is the fundamental issue in the code. We have
                    // a parameters that is initialized first and synchronously. This
                    // means that we should work on the best way to make the code.
                    //
                    // We should make sure that this works in the best way possible.
                    //
                    // The flow is `SchedulingEnvironment` -> `Parameters` -> `Solution`
                    //
                    // This flow means that the if the `Solution` is inconsistent with the
                    // `Parameters` that is okay, but not the other way around.
                    parameters
                        .operational_ids
                        .iter()
                        .filter(|e| e.1.contains(&sp.resource))
                        .map(|e| ((e.clone(), (*won, *acn)), Delegate::Assess))
                })
            })
            .collect();

        let objective_value = Self::ObjectiveValue::default();

        Self {
            objective_value,
            operational_state_machine,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue)
    {
        self.objective_value = other_objective_value;
    }
}

impl<Ss> SwapSolution<Ss> for SupervisorSolution
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    fn swap(id: &Id, solution: Self, system_solution: &mut Ss)
    {
        system_solution.supervisor_swap(id, solution);
    }
}
/// The SupervisorSolution is a state machine that keeps track of all the
/// states of the operational agents. It is a solution representation of
/// a **iterative combinatorial auction algorithms**.
///
/// We should be careful about how we implement this system.
impl SupervisorSolution
{
    pub fn turn_work_order_into_delegate_assess(&mut self, work_order_number: WorkOrderNumber)
    {
        self.operational_state_machine
            .iter_mut()
            .filter(|(key, _)| key.1.0 == work_order_number)
            .for_each(|(_, delegate)| *delegate = Delegate::Assess)
    }

    pub fn count_unique_woa(&self) -> usize
    {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .len()
    }

    pub fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity>
    {
        self.operational_state_machine
            .iter()
            .filter(|(_, val)| val.is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    // You have to be afraid of these kind of things here. I believe that the
    // best approach here is to make something that will allow us to work on the
    // We want to make this so that the code works on. The code should send the
    // error to the orchestrator
    pub fn operational_status_by_work_order_activity<Ss>(
        &self,
        work_order_activity: &WorkOrderActivity,
        // It can only ever reference the `loaded_shared_solution`
        loaded_shared_solution: &Guard<Arc<Ss>>,
    ) -> Result<Vec<(Id, Delegate, MarginalFitness)>>
    where
        Ss: SystemSolutions,
    {
        let mut out = Vec::new();

        for (id_woa, delegate) in &self.operational_state_machine {
            if id_woa.1 != *work_order_activity {
                continue;
            }

            // How should we handle this edge case? I think that the most important
            // thing here is to make the system. Able to function... The real issue
            // is that there is state present in `self.operational_state_machine` that
            // is not available in the operational_actor_solutions.
            //
            // The issue here is whether we should accept the delay here and give the
            // responsibility for eventual consistency to the higher levels.
            //
            // Okay, this can introduce really annoying bugs into the system. But that said
            // this is still a non.
            let op = match loaded_shared_solution
                .operational_actor_solutions(&id_woa.0)
                .ok()
            {
                Some(solution) => {
                    solution.marginal_fitness_for_operational_actor(work_order_activity)
                }
                None => continue,
            };

            if let Some(fitness) = op {
                out.push((id_woa.0.clone(), delegate.clone(), fitness.clone()));
            }
        }

        Ok(out)
    }

    pub(crate) fn get_iter(
        &self,
    ) -> std::collections::hash_map::Iter<(Id, WorkOrderActivity), Delegate>
    {
        self.operational_state_machine.iter()
    }

    pub(crate) fn get_assigned_and_unassigned_work_orders(&self) -> Vec<WorkOrderNumber>
    {
        self.operational_state_machine
            .iter()
            .filter(|(_, delegate)| {
                **delegate == Delegate::Assign || **delegate == Delegate::Unassign
            })
            .map(|(id_woa, _)| id_woa.1.0)
            .collect()
    }

    pub(crate) fn get_work_order_activities(&self) -> HashSet<WorkOrderActivity>
    {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect()
    }
}
