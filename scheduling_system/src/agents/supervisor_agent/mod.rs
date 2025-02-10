pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use self::algorithm::SupervisorAlgorithm;
use super::Agent;
use algorithm::delegate::Delegate;
use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::SeedableRng;
use shared_types::supervisor::SupervisorRequestMessage;
use shared_types::supervisor::SupervisorResponseMessage;

#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;

impl Agent<SupervisorAlgorithm, SupervisorRequestMessage, SupervisorResponseMessage> {
    fn update_supervisor_solution_and_parameters(&mut self) -> Result<()> {
        let entering_work_orders_from_strategic = self
            .algorithm
            .loaded_shared_solution
            .strategic
            .supervisor_work_orders_from_strategic(
                &self.algorithm.supervisor_parameters.supervisor_periods,
            );

        self.algorithm
            .supervisor_solution
            .remove_leaving_work_order_activities(&entering_work_orders_from_strategic);

        let locked_scheduling_environment = self
            .scheduling_environment
            .lock()
            .expect("Could not acquire SchedulingEnvironment lock");

        let work_order_activities: Vec<_> = locked_scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(won, _)| entering_work_orders_from_strategic.contains(won))
            .flat_map(|(won, wo)| wo.operations.keys().map(move |acn| (*won, *acn)))
            .collect();

        for work_order_activity in work_order_activities {
            self.algorithm
                .supervisor_parameters
                .create_and_insert_supervisor_parameter(
                    &locked_scheduling_environment,
                    &work_order_activity,
                );

            for operational_agent in self.algorithm.loaded_shared_solution.operational.keys() {
                if operational_agent.1.contains(
                    &self
                        .algorithm
                        .supervisor_parameters
                        .supervisor_parameter(&work_order_activity)
                        .context("The SupervisorParameter was not found")?
                        .resource,
                ) {
                    let operation = locked_scheduling_environment.operation(&work_order_activity);
                    let delegate = Delegate::build(operation);
                    self.algorithm
                        .supervisor_solution
                        .insert_supervisor_solution(
                            operational_agent,
                            delegate,
                            work_order_activity,
                        )
                        .context("Supervisor could not insert operational solution correctly")?;
                }
            }
        }
        Ok(())
    }
}

pub struct SupervisorOptions {
    number_of_unassigned_work_orders: usize,
    rng: StdRng,
}

impl Default for SupervisorOptions {
    fn default() -> Self {
        Self {
            number_of_unassigned_work_orders: 25,
            rng: StdRng::from_os_rng(),
        }
    }
}
