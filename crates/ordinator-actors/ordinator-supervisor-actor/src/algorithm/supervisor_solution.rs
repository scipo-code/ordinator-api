use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use arc_swap::Guard;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use super::supervisor_parameters::SupervisorParameters;
use crate::SupervisorOptions;

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
    type Options = SupervisorOptions;
    type Parameters = SupervisorParameters;

    fn new(parameters: &Self::Parameters, options: &Self::Options) -> Self
    {
        // The SupervisorParameters should have knowledge of the agents.

        let operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate> = parameters
            .supervisor_work_orders
            .iter()
            .flat_map(|(won, inner)| {
                inner.iter().flat_map(|(acn, sp)| {
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
    //
    pub fn operational_status_by_work_order_activity<Ss>(
        &self,
        work_order_activity: &WorkOrderActivity,
        // It can only ever reference the `loaded_shared_solution`
        loaded_shared_solution: &Guard<Arc<Ss>>,
    ) -> Vec<(Id, Delegate, MarginalFitness)>
    where
        Ss: SystemSolutionTrait,
    {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| id_woa.1 == *work_order_activity)
            .map(|(id_woa, del)| {
                (
                    id_woa.0.clone(),
                    del,
                    // This is what you want to return. I do not think that
                    // there is a better approach. You should simply extract
                    // the most simple part of this.
                    loaded_shared_solution
                        .operational(&id_woa.0)
                        .marginal_fitness_for_operational_actor(work_order_activity),
                )
            })
            .filter(|id_del_opt_mar_fit| id_del_opt_mar_fit.2.is_some())
            .map(|id_del_opt_mar_fit| {
                (
                    id_del_opt_mar_fit.0,
                    id_del_opt_mar_fit.1.clone(),
                    id_del_opt_mar_fit.2.cloned().unwrap(),
                )
            })
            .collect()
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
