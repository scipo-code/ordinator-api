pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use algorithm::assert_functions::OperationalAlgorithmAsserts;
use algorithm::{
    operational_parameter::OperationalParameter,
    operational_solution::{Assignment, MarginalFitness, OperationalAssignment},
};
use anyhow::{Context, Result};
use assert_functions::OperationalAssertions;
use std::sync::{Arc, Mutex};

use shared_types::operational::{
    OperationalConfiguration, OperationalRequestMessage, OperationalResponseMessage,
};
use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
use shared_types::scheduling_environment::work_order::{
    operation::Work, WorkOrderActivity, WorkOrderNumber,
};
use shared_types::scheduling_environment::worker_environment::resources::Id;

use shared_types::scheduling_environment::{
    work_order::operation::Operation, SchedulingEnvironment,
};

use tracing::{event, Level};

use crate::agents::{supervisor_agent::algorithm::delegate::Delegate, OperationalSolution};

use self::algorithm::OperationalAlgorithm;

use super::orchestrator::NotifyOrchestrator;
use super::traits::ActorBasedLargeNeighborhoodSearch;
use super::{Agent, AgentMessage, ScheduleIteration};

impl Agent<OperationalAlgorithm, OperationalRequestMessage, OperationalResponseMessage> {
    pub fn create_operational_parameter(
        &mut self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<()> {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();

        let operation: &Operation = scheduling_environment.operation(work_order_activity);

        assert!(
            operation.work_remaining() > &Some(Work::from(0.0))
                || self
                    .algorithm
                    .loaded_shared_solution
                    .supervisor
                    .operational_state_machine
                    .get(&(self.agent_id.clone(), *work_order_activity))
                    .unwrap()
                    .is_done()
        );

        // TODO: move this around
        let operational_parameter = OperationalParameter::new(
            operation.work_remaining().unwrap(),
            operation.operation_analytic.preparation_time,
        );

        self.algorithm
            .insert_operational_parameter(*work_order_activity, operational_parameter);

        self.algorithm
            .history_of_dropped_operational_parameters
            .insert(*work_order_activity);

        Ok(())
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        let _schedule_iteration = ScheduleIteration::default();
        let start_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::Beginning,
            &self.algorithm.availability,
        );

        let end_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::End,
            &self.algorithm.availability,
        );

        let unavailability_start_event = OperationalAssignment::new(vec![start_event]);

        let unavailability_end_event = OperationalAssignment::new(vec![end_event]);

        self.algorithm
            .operational_solution
            .work_order_activities_assignment
            .push((
                (WorkOrderNumber(0), ActivityNumber(0)),
                unavailability_start_event,
            ));

        self.algorithm
            .operational_solution
            .work_order_activities_assignment
            .push((
                (WorkOrderNumber(0), ActivityNumber(0)),
                unavailability_end_event,
            ));

        let operational_options = OperationalOptions {
            number_of_activities: 10,
        };

        loop {
            self.algorithm.run_lns_iteration(&operational_options)?
        }
        // {
        //         let mut rng = rand::thread_rng();

        //         self.algorithm.load_shared_solution();

        //         // FIX: START
        //         // This should be part of the `schedule` method not handled openly like here.
        //         let loaded_supervisor_solution =
        //             &self.algorithm.loaded_shared_solution.supervisor;

        //         for (work_order_activity, delegate) in
        //             loaded_supervisor_solution.state_of_agent(&self.operational_id)
        //         {
        //             if delegate == Delegate::Done {
        //                 continue;
        //             }
        //             self.create_operational_parameter(&work_order_activity)
        //                 .expect("Could not create OperationalParameter");
        //         }

        //         self.algorithm
        //             .remove_delegate_drop(&self.operational_id);

        //         // FIX: END

        //         // FIX
        //         // All kinds of event debugging should be used for
        //         // event!(
        //         //     Level::DEBUG,
        //         //     operational_solutions = self
        //         //         .algorithm
        //         //         .operational_solution
        //         //         .work_order_activities_assignment
        //         //         .len(),
        //         //     operational_parameters = self
        //         //         .algorithm
        //         //         .operational_parameters
        //         //         .work_order_parameters
        //         //         .len()
        //         // );
        //         // FIX

        //         let temporary_operational_solution: OperationalSolution =
        //             self.algorithm.operational_solution.clone();

        //         self.algorithm
        //             .unschedule(OperationalOptions {
        //                 number_of_activities: 10,
        //             })
        //             .context("Random work orders could not be unscheduled")?;

        //         self.algorithm
        //             .schedule()
        //             .expect("Operational.schedule() method failed");

        //         let is_better_schedule = self
        //             .algorithm
        //             .calculate_objective_value()
        //             .with_context(|| format!("{:#?}", schedule_iteration))
        //             .expect("Error ");

        //         self.assert_marginal_fitness_is_correct()
        //             .with_context(|| {
        //                 format!(
        //                     "\n{}: {}\n\t{:?}\n\t{}\n\tIncorrect {}",
        //                     std::any::type_name::<OperationalAgent>()
        //                         .split("::")
        //                         .last()
        //                         .unwrap()
        //                         .bright_red(),
        //                     self.operational_id.to_string().bright_blue(),
        //                     schedule_iteration,
        //                     format!(
        //                         "Number of {}: {}",
        //                         std::any::type_name::<OperationalSolution>()
        //                             .split("::")
        //                             .last()
        //                             .unwrap(),
        //                         self.algorithm
        //                             .operational_solution
        //                             .work_order_activities_assignment
        //                             .len(),
        //                     )
        //                     .bright_yellow(),
        //                     std::any::type_name::<MarginalFitness>()
        //                         .split("::")
        //                         .last()
        //                         .unwrap()
        //                         .bright_purple(),
        //                 )
        //             })
        //             .expect(&format!(
        //                 "Error in the {}",
        //                 std::any::type_name::<MarginalFitness>()
        //                     .split("::")
        //                     .last()
        //                     .unwrap()
        //             ));

        //         if is_better_schedule {
        //             self.algorithm
        //                 .make_atomic_pointer_swap(&self.operational_id);
        //             self.algorithm.load_shared_solution();
        //             assert_eq!(
        //                 &self.algorithm.operational_solution,
        //                 self.algorithm
        //                     .loaded_shared_solution
        //                     .operational
        //                     .get(&self.operational_id)
        //                     .unwrap()
        //             );
        //         } else {
        //             self.algorithm.operational_solution = temporary_operational_solution;

        //             event!(Level::INFO, operational_objective_value = ?self.algorithm.operational_solution.objective_value);
        //         };

        //         // WARN: You cannot assert the objective here! The operational agent actually has two different
        //         ctx.wait(
        //             tokio::time::sleep(tokio::time::Duration::from_millis(
        //                 dotenvy::var("OPERATIONAL_THROTTLING")
        //                     .expect("The OPERATIONAL_THROTTLING environment variable should always be set")
        //                     .parse::<u64>()
        //                     .expect("The OPERATIONAL_THROTTLING environment variable have to be an u64 compatible type"),
        //             ))
        //             .into_actor(self),
        //         );
        //         self.algorithm
        //             .assert_no_operation_overlap()
        //             .with_context(|| {
        //                 format!(
        //                     "OperationalAgent: {} is having overlaps in his state",
        //                     self.operational_id
        //                 )
        //             })
        //             .expect("");

        //         ctx.notify(ScheduleIteration {
        //             loop_iteration: schedule_iteration.loop_iteration + 1,
        //         });
        //         Ok(())
        //     }
    }
}

pub struct OperationalOptions {
    number_of_activities: usize,
}
