use anyhow::{ensure, Result};

use super::OperationalAlgorithm;

pub trait OperationalAlgorithmAsserts {
    fn assert_no_operation_overlap(&self) -> Result<()>;
}

impl OperationalAlgorithmAsserts for OperationalAlgorithm {
    fn assert_no_operation_overlap(&self) -> Result<()> {
        let operational_solutions = self
            .operational_solution
            .work_order_activities_assignment
            .iter()
            .flat_map(|woa_os| woa_os.1.assignments.clone())
            .chain(self.operational_non_productive.0.clone());

        for (index_1, operational_solution_1) in operational_solutions.clone().enumerate() {
            for (index_2, operational_solution_2) in operational_solutions.clone().enumerate() {
                if index_1 == index_2 {
                    continue;
                }
                ensure!(
                    !(operational_solution_1.start > operational_solution_2.finish
                        && operational_solution_2.finish > operational_solution_1.start),
                    format!(
                        "{:?}\n : {:?}\n is overlapping with \n{:?}\n : {:?}",
                        operational_solution_1,
                        operational_solution_1,
                        operational_solution_2,
                        operational_solution_2
                    )
                );
            }
        }
        Ok(())
    }
}
