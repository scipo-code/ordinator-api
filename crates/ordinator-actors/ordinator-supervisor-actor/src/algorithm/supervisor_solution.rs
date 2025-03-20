use std::collections::HashMap;
use std::collections::HashSet;

use shared_types::scheduling_environment::work_order::WorkOrderActivity;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::Id;

use super::delegate::Delegate;
use crate::agents::OperationalSolution;
use crate::agents::SupervisorSolution;
use crate::agents::operational_agent::algorithm::operational_solution::MarginalFitness;

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct SupervisorSolution
{
    pub objective_value: SupervisorObjectiveValue,
    operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate>,
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

    pub fn operational_status_by_work_order_activity<'a>(
        &self,
        work_order_activity: &WorkOrderActivity,
        operational_solutions: &'a HashMap<Id, OperationalSolution>,
    ) -> Vec<(Id, Delegate, &'a MarginalFitness)>
    {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| id_woa.1 == *work_order_activity)
            .map(|(id_woa, del)| {
                (
                    id_woa.0.clone(),
                    *del,
                    operational_solutions
                        .get(&id_woa.0)
                        .expect("The agent should be present")
                        .scheduled_work_order_activities
                        .iter()
                        .find(|woa_ass| woa_ass.0 == id_woa.1)
                        .map(|woa_ass| &woa_ass.1.marginal_fitness), /* What should be done if
                                                                      * the Agent does not yet
                                                                      * have a */
                )
            })
            .filter(|id_del_opt_mar_fit| id_del_opt_mar_fit.2.is_some())
            .map(|id_del_opt_mar_fit| {
                (
                    id_del_opt_mar_fit.0,
                    id_del_opt_mar_fit.1,
                    id_del_opt_mar_fit.2.unwrap(),
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

    pub fn delegates_for_agent(
        &self,
        operational_agent: &Id,
    ) -> HashMap<WorkOrderActivity, Delegate>
    {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| &id_woa.0 == operational_agent)
            .map(|(id_woa, del)| (id_woa.1, *del))
            .collect()
    }

    pub fn count_delegate_types(&self, operational_agent: &Id) -> (u64, u64, u64)
    {
        let mut count_assign = 0;
        let mut count_assess = 0;
        let mut count_unassign = 0;
        for delegate in self.delegates_for_agent(operational_agent).values() {
            match delegate {
                Delegate::Assess => count_assess += 1,
                Delegate::Assign => count_assign += 1,
                Delegate::Unassign => count_unassign += 1,
                Delegate::Drop => (),
                Delegate::Done => (),
                Delegate::Fixed => (),
            }
        }
        (count_assign, count_assess, count_unassign)
    }
}
