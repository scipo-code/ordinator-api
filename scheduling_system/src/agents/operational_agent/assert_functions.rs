use anyhow::{ensure, Result};

use super::OperationalAgent;
use crate::agents::supervisor_agent::delegate::Delegate;

#[allow(dead_code)]
pub trait OperationalAssertions {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>;
    fn assert_no_operation_overlap(&self) -> Result<()>;
}

impl OperationalAssertions for OperationalAgent {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()> {
        for delegate in self
            .operational_algorithm
            .loaded_shared_solution
            .supervisor
            .state_of_agent(&self.operational_id)
            .values()
        {
            ensure!(delegate != &Delegate::Unassign)
        }

        Ok(())
    }

    fn assert_no_operation_overlap(&self) -> Result<()> {
        for (index_1, operational_solution_1) in self
            .operational_algorithm
            .operational_solution
            .work_order_activities
            .iter()
            .enumerate()
        {
            for (index_2, operational_solution_2) in self
                .operational_algorithm
                .operational_solution
                .work_order_activities
                .iter()
                .enumerate()
            {
                if index_1 == index_2 {
                    continue;
                }
                ensure!(
                    !(operational_solution_1.1.start_time()
                        > operational_solution_2.1.finish_time()
                        && operational_solution_2.1.finish_time()
                            > operational_solution_1.1.start_time()),
                    format!(
                        "{:?}\n : {:?}\n is overlapping with \n{:?}\n : {:?}",
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
