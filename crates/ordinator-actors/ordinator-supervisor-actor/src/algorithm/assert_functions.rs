use std::collections::HashSet;

use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use tracing::Level;
use tracing::event;

use super::SupervisorSolution;

#[allow(dead_code)]
pub trait OperationalStateMachineAssertions {
    fn assert_that_unassigned_woas_are_valid(&self);
    fn assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess(
        &self,
    );
}

impl OperationalStateMachineAssertions for SupervisorSolution {
    fn assert_that_unassigned_woas_are_valid(&self) {
        for work_order_activity in self
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .collect::<Vec<_>>()
        {
            // What is it that is mutable here?
            let mut delegates_by_woa = self
                .operational_state_machine
                .iter()
                .filter(|(key, _)| key.1 == *work_order_activity)
                .map(|(_, delegate)| delegate);

            let is_all_assess =
                delegates_by_woa.all(|delegate| delegate.is_assess() || delegate.is_done());

            let is_all_drop =
                delegates_by_woa.all(|delegate| delegate.is_drop() || delegate.is_done());

            if !(is_all_drop || is_all_assess) {
                event!(Level::ERROR, delegate_by_woa = ?delegates_by_woa);
                panic!("check the 'delegate_by_woa' ERROR in the logs");
            }
        }
    }

    fn assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess(
        &self,
    ) {
        let work_order_numbers: HashSet<WorkOrderNumber> = self
            .operational_state_machine
            .keys()
            .map(|d| d.1.0)
            .collect();

        for work_order_number in work_order_numbers {
            let mut assess_work_orders: HashSet<WorkOrderNumber> = HashSet::new();
            let mut assign_unassign_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

            self.operational_state_machine
                .iter()
                .filter(|(id_woa, _)| id_woa.1.0 == work_order_number)
                .for_each(|osm| {
                    let delegate = osm.1;

                    // Which one is the correct one to use? I think that the right one is th e
                    if *delegate == Delegate::Assess {
                        assess_work_orders.insert(work_order_number);
                    } else if *delegate == Delegate::Assign || *delegate == Delegate::Unassign {
                        assign_unassign_work_orders.insert(work_order_number);
                    }
                });

            assert!(!assess_work_orders.is_empty() || !assign_unassign_work_orders.is_empty());
        }
    }
}

// Your biggest issue at the moment is that you do not understand visibility
// mechanics well enough to simply design the system as you want. That is
// causing a lot of problems. You should never have refactored all that
// code into different crates you should simply have learned to use
// visibility `pub`, `pub(crate)`, and `pub(super)` instead. The fact that
// you did not chase your own blindspots when it comes to knowledge has
// cost you months of wasted development effort.
