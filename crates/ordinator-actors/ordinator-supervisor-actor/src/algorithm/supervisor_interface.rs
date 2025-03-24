use std::collections::HashMap;
use std::collections::HashSet;

use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use super::supervisor_solution::SupervisorSolution;

/// I think that you have to either make this in the `supervisor` agent or
/// in the
impl SupervisorInterface for SupervisorSolution
{
    fn delegates_for_agent(&self, operational_agent: &Id) -> HashMap<WorkOrderActivity, Delegate>
    {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| &id_woa.0 == operational_agent)
            .map(|(id_woa, del)| (id_woa.1, *del))
            .collect()
    }

    fn delegated_tasks(&self, operational_agent: &Id) -> HashSet<WorkOrderActivity>
    {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, del)| id_woa.0 == self.id && (del.is_assign() || del.is_assess()))
            .map(|(id_woa, _)| id_woa.1)
            .collect::<HashSet<_>>()
    }
}
