use anyhow::Result;
use anyhow::ensure;
use chrono::TimeDelta;
use colored::Colorize;

use super::algorithm::OperationalNonProductive;
use super::algorithm::operational_parameter::OperationalParameters;
use crate::agents::Algorithm;
use crate::agents::OperationalSolution;
use crate::agents::operational_agent::algorithm::operational_events::OperationalEvents;
use crate::agents::operational_agent::algorithm::operational_solution::MarginalFitness;
use crate::agents::supervisor_agent::algorithm::delegate::Delegate;

#[allow(dead_code)]
pub trait OperationalAssertions
{
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>;
    fn assert_marginal_fitness_is_correct(&self) -> Result<()>;
}

impl OperationalAssertions
    for Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive>
{
    fn assert_operational_solutions_does_not_have_delegate_unassign(&self) -> Result<()>
    {
        for delegate in self
            .loaded_shared_solution
            .supervisor
            .delegates_for_agent(&self.id)
            .values()
        {
            ensure!(delegate != &Delegate::Unassign)
        }

        Ok(())
    }

    fn assert_marginal_fitness_is_correct(&self) -> Result<()>
    {
        for assignments in self.solution.scheduled_work_order_activities.windows(3) {
            let finish_of_prev = assignments[0].1.finish_time();
            let start_of_next = assignments[2].1.start_time();
            let combined_non_productive: TimeDelta = self
                .solution_intermediate
                .0
                .iter()
                .filter(|non_prod| match &non_prod.event_type {
                    OperationalEvents::NonProductiveTime(_time_interval) => {
                        finish_of_prev <= non_prod.start && non_prod.finish <= start_of_next
                    }
                    _ => false,
                })
                .map(|non_prod| (non_prod.finish - non_prod.start))
                .sum();

            ensure!(
                assignments[1].1.marginal_fitness
                    == MarginalFitness::Scheduled(combined_non_productive.num_seconds() as u64),
                format!(
                    "{}\n{}\n\n{}\n{}\n\n{}\n{}\n{}",
                    format!("{:<10}: {:?}", "Activity", assignments[1].0)
                        .to_string()
                        .bright_yellow(),
                    format!(
                        "{:<10}: {:?}",
                        "Work hours",
                        self.parameters
                            .work_order_parameters
                            .get(&assignments[1].0)
                            .unwrap()
                            .work
                    )
                    .bright_yellow(),
                    format!("{:<18}: {:?}", "Actual", assignments[1].1.marginal_fitness)
                        .bright_purple(),
                    format!(
                        "{:<18}: {:?}",
                        "Calculated first",
                        MarginalFitness::Scheduled(combined_non_productive.num_seconds() as u64)
                    )
                    .bright_purple(),
                    format!(
                        "Previous Activity: {:?}\n{:<10}: {:#?}\n{:<10}: {:#?}\n",
                        assignments[0].0,
                        "Start at",
                        assignments[0].1.start_time(),
                        "Finish at",
                        assignments[0].1.finish_time(),
                    )
                    .bright_green(),
                    format!(
                        "Current Activity: {:?}\n{:<10}: {:#?}\n{:<10}: {:#?}\n",
                        assignments[1].0,
                        "Start at",
                        assignments[1].1.start_time(),
                        "Finish at",
                        assignments[1].1.finish_time(),
                    )
                    .bright_green(),
                    format!(
                        "Next Activity: {:?}\n{:<10}: {:#?}\n{:<10}: {:#?}\n",
                        assignments[2].0,
                        "Start at",
                        assignments[2].1.start_time(),
                        "Finish at",
                        assignments[2].1.finish_time(),
                    )
                    .bright_green(),
                ),
            );
        }
        Ok(())
    }
}
