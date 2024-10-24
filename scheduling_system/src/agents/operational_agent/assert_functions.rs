use std::sync::atomic::Ordering;

use shared_types::scheduling_environment::work_order::WorkOrderActivity;

use crate::agents::supervisor_agent::delegate::Delegate;

use super::OperationalAgent;

pub trait OperationalAssertions {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self);
}

impl OperationalAssertions for OperationalAgent {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) {
        let work_order_solutions = self
            .operational_algorithm
            .operational_solutions
            .0
            .iter()
            .map(|(woa, _)| woa)
            .cloned()
            .collect::<Vec<WorkOrderActivity>>();

        self.operational_algorithm
            .operational_parameters
            .0
            .keys()
            .for_each(|woa| {
                if self
                    .operational_algorithm
                    .operational_parameters
                    .0
                    .get(woa)
                    .unwrap()
                    .delegated
                    .load(Ordering::SeqCst)
                    == Delegate::Unassign
                {
                    assert!(!work_order_solutions.contains(woa));
                }
            });
    }
}
