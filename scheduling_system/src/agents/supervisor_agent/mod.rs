pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use algorithm::delegate::Delegate;
use anyhow::{Context, Result};
use assert_functions::SupervisorAssertions;
use rand::{prelude::SliceRandom, rngs::ThreadRng};
use shared_types::supervisor::SupervisorResponseMessage;

use shared_types::supervisor::SupervisorRequestMessage;

use tracing::{event, instrument, Level};

use self::algorithm::SupervisorAlgorithm;

use super::{
    orchestrator::NotifyOrchestrator, traits::ActorBasedLargeNeighborhoodSearch, Agent,
    ArcSwapSharedSolution, ScheduleIteration,
};

impl Agent<SupervisorAlgorithm, SupervisorRequestMessage, SupervisorResponseMessage> {
    fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: u64,
        mut rng: ThreadRng,
    ) -> Result<()> {
        let work_order_numbers = self
            .algorithm
            .supervisor_solution
            .get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(&mut rng, number_of_work_orders as usize)
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.algorithm
                .unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule work_order_number: {:?}",
                        work_order_number
                    )
                })?;
        }
        Ok(())
        // self.algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&old_state).unwrap();
    }

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

    pub(crate) fn run(&mut self) -> Result<()> {
        // self.assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution()
        // .unwrap();

        let options = SupervisorOptions {};

        loop {
            self.algorithm.run_lns_iteration(options)
        }

        // {
        //         self.algorithm.load_shared_solution();
        //         self.update_supervisor_solution_and_parameters()
        //             .expect("Could not load the data from the load SharedSolution");

        //         self.assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution()
        //             .expect("OperationalStates should correspond with TacticalOperations");

        //         event!(
        //             Level::DEBUG,
        //             number_of_operational_states = self.algorithm.supervisor_solution.len()
        //         );

        //         event!(
        //             Level::DEBUG,
        //             number_of_operational_agents = ?self.number_of_operational_agents
        //         );

        //         let rng = rand::thread_rng();
        //         self.algorithm.calculate_objective_value();

        //         let old_supervisor_solution = self.algorithm.supervisor_solution.clone();

        //         let number_of_removed_work_orders = 10;
        //         self.unschedule_random_work_orders(number_of_removed_work_orders, rng)
        //             .unwrap_or_else(|err| {
        //                 panic!(
        //                     "Error: {}, Could not destroy {}",
        //                     err,
        //                     std::any::type_name::<SupervisorSolution>()
        //                 )
        //             });

        //         self.algorithm
        //             .schedule()
        //             .expect("SupervisorAlgorithm.schedule method failed");
        //         // self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();

        //         let new_objective_value = self.algorithm.calculate_objective_value();

        //         assert_eq!(
        //             new_objective_value,
        //             self.algorithm.calculate_objective_value()
        //         );

        //         // self.algorithm.operational_state_machine.assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess();
        //         // self.algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&current_state).unwrap();

        //         if self.algorithm.objective_value >= old_supervisor_solution.objective_value {
        //             self.algorithm.make_atomic_pointer_swap();
        //         } else if self.algorithm.objective_value
        //             < old_supervisor_solution.objective_value
        //         {
        //             assert!(
        //                 self.algorithm.objective_value
        //                     >= old_supervisor_solution.objective_value
        //             );
        //             self.algorithm.supervisor_solution = old_supervisor_solution;
        //             self.algorithm.calculate_objective_value();
        //         }

        //         event!(
        //             Level::INFO,
        //             supervisor_objective_value = self.algorithm.objective_value
        //         );

        //         ctx.wait(
        //             tokio::time::sleep(tokio::time::Duration::from_millis(
        //                 dotenvy::var("SUPERVISOR_THROTTLING")
        //                     .expect("The SUPERVISOR_THROTTLING environment variable should always be set")
        //                     .parse::<u64>()
        //                     .expect("The SUPERVISOR_THROTTLING environment variable have to be an u64 compatible type"),
        //             ))
        //             .into_actor(self),
        //         );
        //         ctx.notify(ScheduleIteration {
        //             loop_iteration: schedule_iteration.loop_iteration + 1,
        //         });
        //         Ok(())
        //     }
    }
}

pub struct SupervisorOptions {}
