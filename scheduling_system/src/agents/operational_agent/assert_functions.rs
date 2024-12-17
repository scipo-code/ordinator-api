use anyhow::{ensure, Result};
use chrono::TimeDelta;
use colored::Colorize;

use super::OperationalAgent;
use crate::agents::{
    operational_agent::algorithm::operational_solution::MarginalFitness,
    supervisor_agent::algorithm::delegate::Delegate,
};

#[allow(dead_code)]
pub trait OperationalAssertions {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>;
    fn assert_no_operation_overlap(&self) -> Result<()>;
    fn assert_marginal_fitness_is_correct(&self) -> Result<()>;
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
            .work_order_activities_assignment
            .iter()
            .enumerate()
        {
            for (index_2, operational_solution_2) in self
                .operational_algorithm
                .operational_solution
                .work_order_activities_assignment
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

    fn assert_marginal_fitness_is_correct(&self) -> Result<()> {
        for window in self
            .operational_algorithm
            .operational_solution
            .work_order_activities_assignment
            .windows(3)
        {
            let finish_of_prev = window[0].1.finish_time();
            let start_of_next = window[2].1.start_time();
            let combined_non_productive: TimeDelta = self
                .operational_algorithm
                .operational_non_productive
                .0
                .iter()
                .filter(|non_prod| {
                    finish_of_prev <= non_prod.start || non_prod.finish <= start_of_next
                })
                .map(|non_prod| (non_prod.finish - non_prod.start))
                .sum();

            ensure!(
                window[1].1.marginal_fitness
                    == MarginalFitness::Fitness(combined_non_productive.num_seconds() as u64),
                format!(
                    "{}\nActual: {}\nCalculated: {}\n{}\n{}",
                    format!("{:#?}", window[1].0).to_string().bright_yellow(),
                    format!("{:?}", window[1].1.marginal_fitness).bright_purple(),
                    format!(
                        "{:?}",
                        MarginalFitness::Fitness(combined_non_productive.num_seconds() as u64)
                    )
                    .bright_purple(),
                    format!(
                        "{:?}",
                        self.operational_algorithm
                            .operational_parameters
                            .work_order_parameters
                            .get(&window[1].0)
                            .unwrap()
                            .work
                    )
                    .bright_yellow(),
                    format!(
                        "Start at: {:#?}, finish at: {:#?}",
                        window[2].1.start_time(),
                        window[2].1.finish_time(),
                    )
                    .bright_green()
                ),
            );
        }
        Ok(())
    }
}
