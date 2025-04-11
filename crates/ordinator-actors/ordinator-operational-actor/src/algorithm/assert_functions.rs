use anyhow::Result;
use anyhow::ensure;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;

use super::OperationalAlgorithm;
use super::OperationalNonProductive;
use super::operational_parameter::OperationalParameters;
use super::operational_solution::OperationalSolution;
use crate::algorithm::Algorithm;

pub trait OperationalAlgorithmAsserts
{
    fn assert_no_operation_overlap(&self) -> Result<()>;
}

// TODO [ ]
// These kind of asserts should also look at the `SharedSolution`
impl<Ss> OperationalAlgorithmAsserts for OperationalAlgorithm<Ss>
where
    Ss: SharedSolutionTrait,
{
    fn assert_no_operation_overlap(&self) -> Result<()>
    {
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
