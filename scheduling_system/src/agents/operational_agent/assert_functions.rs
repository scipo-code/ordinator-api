use std::sync::atomic::Ordering;

use anyhow::{ensure, Result};
use shared_types::scheduling_environment::work_order::WorkOrderActivity;

use super::OperationalAgent;
use crate::agents::supervisor_agent::delegate::Delegate;

#[allow(dead_code)]
pub trait OperationalAssertions {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>;
    fn assert_no_operation_overlap(&self) -> Result<()>;
}

impl OperationalAssertions for OperationalAgent {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()> {
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

        Ok(())
    }

    fn assert_no_operation_overlap(&self) -> Result<()> {
        for (_, operational_solution_1) in self
            .operational_algorithm
            .operational_solutions
            .0
            .iter()
            .enumerate()
        {
            for (_, operational_solution_2) in self
                .operational_algorithm
                .operational_solutions
                .0
                .iter()
                .enumerate()
            {
                ensure!(
                    operational_solution_1.1.start_time() > operational_solution_2.1.finish_time()
                        && operational_solution_2.1.finish_time()
                            > operational_solution_1.1.start_time(),
                    format!(
                        "{:?} : {:?} is overlapping with {:?} : {:?}",
                        operational_solution_1.0,
                        operational_solution_1.1,
                        operational_solution_2.0,
                        operational_solution_2.1
                    )
                );
            }
        }
        Ok(())
    }
}
