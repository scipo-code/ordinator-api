pub mod algorithm;
pub mod message_handlers;

use algorithm::assert_functions::TacticalAssertions;
use anyhow::{Context, Result};
use shared_types::tactical::tactical_response_status::TacticalResponseStatus;
use shared_types::tactical::{TacticalRequestMessage, TacticalResponseMessage};

use crate::agents::tactical_agent::algorithm::TacticalAlgorithm;

use super::traits::ActorBasedLargeNeighborhoodSearch;
use super::{Agent, ScheduleIteration};

impl Agent<TacticalAlgorithm, TacticalRequestMessage, TacticalResponseMessage> {
    pub fn status(&self) -> Result<TacticalResponseStatus> {
        Ok(TacticalResponseStatus::new(
            self.algorithm.objective_value(),
            self.algorithm.tactical_days.clone(),
        ))
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        // self.tactical_algorithm.schedule().with_context(|| format!("Initial call of: {}", std::any::type_name::<TacticalAlgorithm>())).expect("Failed initial schedule call");

        let options = TacticalOptions {};
        let schedule_iteration = ScheduleIteration::default();
        loop {
            self.algorithm.run_lns_iteration(&options)?
        }
        // {
        //         let mut rng = rand::thread_rng();

        //         self.tactical_algorithm.load_shared_solution();

        //         let current_tactical_solution = self.tactical_algorithm.tactical_solution.clone();

        //         self.tactical_algorithm
        //             .unschedule_random_work_orders(&mut rng, 50)
        //             .context("random unschedule failed")
        //             .expect("Error in the Handler<ScheduleIteration>");

        //         self.tactical_algorithm.schedule().with_context(|| format!("{:#?}", schedule_iteration)).expect("TacticalAlgorithm.schedule method failed");

        //         let total_excess_hours = self.tactical_algorithm.asset_that_capacity_is_not_exceeded().ok();

        //         if self.tactical_algorithm.calculate_objective_value().expect("Could not calculate objective value correctly")
        //             < current_tactical_solution.objective_value
        //         {
        //             self.tactical_algorithm
        //                 .make_atomic_pointer_swap();
        //             event!(Level::INFO,
        //                  new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
        //                  tactical_objective_value = ?current_tactical_solution.objective_value,
        //                  difference_in_objective_value = self.tactical_algorithm.tactical_solution.objective_value.0 as i64 - current_tactical_solution.objective_value.0 as i64,
        //                  total_excess_hours = ?total_excess_hours,
        //                  scheduled_work_orders = self
        //                     .tactical_algorithm
        //                     .tactical_solution
        //                     .tactical_scheduled_work_orders.scheduled_work_orders())
        //         } else {
        //             event!(Level::INFO,
        //                  new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
        //                  tactical_objective_value = ?current_tactical_solution.objective_value,
        //                  difference_in_objective_value = self.tactical_algorithm.tactical_solution.objective_value.0 as i64 - current_tactical_solution.objective_value.0 as i64,
        //                  total_excess_hours = ?total_excess_hours,
        //                  scheduled_work_orders = self
        //                 .tactical_algorithm
        //                 .tactical_solution
        //                 .tactical_scheduled_work_orders.scheduled_work_orders(),
        //                 );

        //             self.tactical_algorithm.tactical_solution = current_tactical_solution;
        //         };

        //             event!(Level::INFO,
        //                  new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
        //                  total_excess_hours = ?total_excess_hours,
        //                  scheduled_work_orders = self
        //                 .tactical_algorithm
        //                 .tactical_solution
        //                 .tactical_scheduled_work_orders.scheduled_work_orders());

        //         ctx.wait(
        //             tokio::time::sleep(tokio::time::Duration::from_millis(
        //                 dotenvy::var("TACTICAL_THROTTLING")
        //                     .expect("The TACTICAL_THROTTLING environment variable should always be set")
        //                     .parse::<u64>()
        //                     .expect("The TACTICAL_THROTTLING environment variable have to be an u64 compatible type"),
        //             ))
        //             .into_actor(self),
        //         );
        //         ctx.notify(ScheduleIteration {loop_iteration: schedule_iteration.loop_iteration + 1});
        //         self.tactical_algorithm
        //             .asset_that_loading_matches_scheduled()
        //             .with_context(|| format!("{:#?}", schedule_iteration))
        //             .unwrap();
        //         Ok(())
        //     }
    }
}

pub struct TacticalOptions {}
