use anyhow::{Result, ensure};

use super::operational_solution::OperationalSolution;
use crate::algorithm::Algorithm;

use super::{OperationalNonProductive, operational_parameter::OperationalParameters};

pub trait OperationalAlgorithmAsserts {
    fn assert_no_operation_overlap(&self) -> Result<()>;
}

impl OperationalAlgorithmAsserts
    for Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive>
{
    fn assert_no_operation_overlap(&self) -> Result<()> {
        let operational_solutions = self
            .solution
            .scheduled_work_order_activities
            .iter()
            .flat_map(|woa_os| woa_os.1.assignments.clone())
            .chain(self.solution_intermediate.0.clone());

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
