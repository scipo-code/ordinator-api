use super::SupervisorSolution;
use crate::agents::supervisor_agent::delegate::Delegate;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use std::collections::HashSet;
use tracing::{event, Level};

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
        let work_order_numbers: HashSet<WorkOrderNumber> = self.get_work_order_numbers();
        for work_order_number in work_order_numbers {
            let mut assess_work_orders: HashSet<WorkOrderNumber> = HashSet::new();
            let mut assign_unassign_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

            self.operational_state_machine
                .iter()
                .filter(|(id_woa, _)| id_woa.1 .0 == work_order_number)
                .for_each(|osm| {
                    let delegate = osm.1;

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
