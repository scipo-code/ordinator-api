use std::collections::HashMap;
use std::collections::HashSet;

use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use super::supervisor_solution::SupervisorSolution;

/// I think that you have to either make this in the `supervisor` agent or
/// in the
impl SupervisorInterface for SupervisorSolution {
    fn delegates_for_agent(&self, operational_agent: &Id) -> HashMap<WorkOrderActivity, Delegate> {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, _)| &id_woa.0 == operational_agent)
            .map(|(id_woa, del)| (id_woa.1, del.clone()))
            .collect()
    }

    fn delegated_tasks(&self, operational_agent: &Id) -> HashSet<WorkOrderActivity> {
        self.operational_state_machine
            .iter()
            .filter(|(id_woa, del)| {
                &id_woa.0 == operational_agent && (del.is_assign() || del.is_assess())
            })
            .map(|(id_woa, _)| id_woa.1)
            .collect::<HashSet<_>>()
    }

    // This function has to be moved.
    fn count_delegate_types(&self, operational_agent: &Id) -> (u64, u64, u64) {
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
