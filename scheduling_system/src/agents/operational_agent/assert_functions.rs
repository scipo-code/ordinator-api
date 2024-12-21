use anyhow::{ensure, Result};
use chrono::TimeDelta;
use colored::Colorize;

use super::OperationalAgent;
use crate::agents::{
    operational_agent::algorithm::{
        operational_events::OperationalEvents, operational_solution::MarginalFitness,
    },
    supervisor_agent::algorithm::delegate::Delegate,
};

#[allow(dead_code)]
pub trait OperationalAssertions {
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>;
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

    fn assert_marginal_fitness_is_correct(&self) -> Result<()> {
        for assignments in self
            .operational_algorithm
            .operational_solution
            .work_order_activities_assignment
            .windows(3)
        {
            let mut non_productive_time_intervals = 0;
            let finish_of_prev = assignments[0].1.finish_time();
            let start_of_next = assignments[2].1.start_time();
            let combined_non_productive: TimeDelta = self
                .operational_algorithm
                .operational_non_productive
                .0
                .iter()
                .filter(|non_prod| match &non_prod.event_type {
                    OperationalEvents::NonProductiveTime(time_interval) => {
                        non_productive_time_intervals += time_interval.duration().num_seconds();
                        finish_of_prev <= non_prod.start || non_prod.finish <= start_of_next
                    }
                    _ => false,
                })
                .map(|non_prod| (non_prod.finish - non_prod.start))
                .sum();

            ensure!(
                (assignments[1].1.marginal_fitness
                    == MarginalFitness::Scheduled(combined_non_productive.num_seconds() as u64))
                    && (assignments[1].1.marginal_fitness
                        == MarginalFitness::Scheduled(non_productive_time_intervals as u64)),
                format!(
                    "{}\n{}\n{}\n{}\n{}",
                    format!("{:<10}: {:?}", "Activity", assignments[1].0)
                        .to_string()
                        .bright_yellow(),
                    format!(
                        "{:<10}: {:?}",
                        "Work hours",
                        self.operational_algorithm
                            .operational_parameters
                            .work_order_parameters
                            .get(&assignments[1].0)
                            .unwrap()
                            .work
                    )
                    .bright_yellow(),
                    format!("{:<10}: {:?}", "Actual", assignments[1].1.marginal_fitness)
                        .bright_purple(),
                    format!(
                        "{:<10}: {:?}",
                        "Calculated",
                        MarginalFitness::Scheduled(combined_non_productive.num_seconds() as u64)
                    )
                    .bright_purple(),
                    format!(
                        "{:<10}: {:#?}\n{:<10}: {:#?}",
                        "Start at",
                        assignments[1].1.start_time(),
                        "Finish at",
                        assignments[1].1.finish_time(),
                    )
                    .bright_green(),
                ),
            );
        }
        Ok(())
    }
}
