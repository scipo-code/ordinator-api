pub mod assert_functions;
pub mod operational_events;
pub mod operational_parameter;
pub mod operational_solution;

use std::{collections::HashSet, sync::Arc};

use anyhow::{bail, ensure, Context, Result};
use assert_functions::OperationalAlgorithmAsserts;
use chrono::{DateTime, TimeDelta, Utc};
use itertools::Itertools;
use operational_events::OperationalEvents;
use operational_parameter::{OperationalParameter, OperationalParameters};
use operational_solution::{
    Assignment, MarginalFitness, OperationalAssignment, OperationalFunctions,
};
use rand::seq::IndexedRandom;
use shared_types::{
    agents::operational::TimeInterval,
    scheduling_environment::{
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::availability::Availability,
    },
};
use tracing::{event, Level};

use crate::agents::{
    supervisor_agent::algorithm::delegate::Delegate,
    traits::{ActorBasedLargeNeighborhoodSearch, ObjectiveValueType},
    Algorithm, OperationalSolution, StrategicSolution, TacticalSolution, WhereIsWorkOrder,
};

use super::OperationalOptions;

pub type OperationalObjectiveValue = u64;

#[derive(Clone, Default)]
pub struct OperationalNonProductive(pub Vec<Assignment>);

impl Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive> {
    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<OperationalAssignment>,
    ) -> (DateTime<Utc>, OperationalEvents) {
        if self.parameters.break_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.parameters.break_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (new_current_time, OperationalEvents::Break(time_interval))
        } else if self.parameters.off_shift_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.parameters.off_shift_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::OffShift(self.parameters.off_shift_interval.clone()),
            )
        } else if self.parameters.toolbox_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.parameters.toolbox_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::Toolbox(self.parameters.toolbox_interval.clone()),
            )
        } else {
            let start = *current_time;
            let (time_until_next_event, next_operational_event) =
                self.determine_next_event(current_time);
            let mut new_current_time = *current_time + time_until_next_event;

            if *current_time == new_current_time {
                (
                    new_current_time + next_operational_event.time_delta(),
                    next_operational_event,
                )
            } else {
                if self.parameters.availability.finish_date < new_current_time {
                    new_current_time = self.parameters.availability.finish_date;
                }
                let time_interval = TimeInterval::new(start.time(), new_current_time.time());
                (
                    new_current_time,
                    OperationalEvents::NonProductiveTime(time_interval),
                )
            }
        }
    }

    // This function makes sure that the created event is adjusted to fit the schedule if there has been any manual intervention in the
    // schedule for the OperationalAgent.
    fn determine_time_interval_of_function(
        &mut self,
        next_operation: Option<OperationalAssignment>,
        current_time: &DateTime<Utc>,
        interval: TimeInterval,
    ) -> TimeInterval {
        // What is this code actually trying to do? I think
        let time_interval: TimeInterval = match next_operation {
            Some(operational_solution) => {
                if operational_solution.start_time().date_naive() == current_time.date_naive() {
                    TimeInterval::new(
                        current_time.time(),
                        interval.end.min(operational_solution.start_time().time()),
                    )
                } else {
                    TimeInterval::new(current_time.time(), interval.end)
                }
            }
            None => TimeInterval::new(current_time.time(), interval.end),
        };
        time_interval
    }

    // This is a problem. What should you do about it? I think that the best thing that you can do is move all this
    // into the `schedule` function and handle it while the code is running. That is probably the best call here. I
    // do not see what other way it could be done in a better way.
    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    ) {
        let time_delta_usize = time_delta.num_seconds() as u64;

        self.solution
            .scheduled_work_order_activities
            .iter_mut()
            .find(|oper_sol| oper_sol.0 == work_order_activity_previous)
            .unwrap()
            .1
            .marginal_fitness = MarginalFitness::Scheduled(time_delta_usize);
    }
}

pub enum ContainOrNextOrNone {
    Contain(OperationalAssignment),
    Next(OperationalAssignment),
    None,
}

pub enum Unavailability {
    Beginning,
    End,
}

// FIX
// Some of the methods here should be moved out of the agent. That will be crucial. You have one hour to m
// make this compile again.
// QUESTION
// What should be changed here to make the ABLNS work on the Algorithm again?
impl ActorBasedLargeNeighborhoodSearch
    for Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive>
{
    type Options = OperationalOptions;

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        let operational_shared_solution = self
            .loaded_shared_solution
            .supervisor
            .delegates_for_agent(&self.id);

        self.solution
            .scheduled_work_order_activities
            // We remain all `OperationalSolution` which are not `Delegate::Drop` where
            // the `Delegate` variant is decided by the
            .retain(|(woa, _)| {
                !operational_shared_solution
                    .get(woa)
                    .unwrap_or_else(|| {
                        assert!(*woa == (WorkOrderNumber(0), 0));
                        &Delegate::Assign
                    })
                    .is_drop()
            });
        Ok(true)
    }

    fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution
                .operational
                .insert(self.id.clone(), self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    // If we are going to implement delta evaluation we should remove this part.
    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType<Self::ObjectiveValue>> {
        let operational_events: Vec<Assignment> = self
            .solution
            .scheduled_work_order_activities
            .iter()
            .flat_map(|(_, os)| os.assignments.iter())
            .cloned()
            .collect();

        event!(Level::DEBUG, operational_events_len = ?operational_events.iter().filter(|val| val.event_type.is_wrench_time()).collect::<Vec<_>>().len());

        let all_events = operational_events
            .into_iter()
            .chain(self.solution_intermediate.0.clone())
            .sorted_unstable_by_key(|ass| ass.start);

        let mut current_time = self.parameters.availability.start_date;

        let mut wrench_time: TimeDelta = TimeDelta::zero();
        let mut break_time: TimeDelta = TimeDelta::zero();
        let mut off_shift_time: TimeDelta = TimeDelta::zero();
        let mut toolbox_time: TimeDelta = TimeDelta::zero();
        let mut non_productive_time: TimeDelta = TimeDelta::zero();

        let mut prev_fitness: TimeDelta = TimeDelta::zero();
        let mut next_fitness: TimeDelta = TimeDelta::zero();
        let mut first_fitness: bool = true;
        let mut current_work_order_activity: Option<WorkOrderActivity> = None;

        self.assert_no_operation_overlap()
            .context("Operational_events overlap in the operational objective value calculation")?;

        // What is the problem here?
        // If the work order changes you
        for assignment in all_events.clone() {
            match &assignment.event_type {
                OperationalEvents::WrenchTime((time_interval, work_order_activity)) => {
                    wrench_time += time_interval.duration();
                    current_time += time_interval.duration();
                    current_work_order_activity = match current_work_order_activity {
                        Some(work_order_activity_previous) => {
                            if work_order_activity_previous != *work_order_activity {
                                let marginal_fitness_time_delta = prev_fitness + next_fitness;

                                self.update_marginal_fitness(
                                    work_order_activity_previous,
                                    marginal_fitness_time_delta,
                                );
                                prev_fitness = next_fitness;
                                next_fitness = TimeDelta::zero();
                            }
                            Some(*work_order_activity)
                        }
                        None => {
                            first_fitness = false;
                            Some(*work_order_activity)
                        }
                    }
                }
                OperationalEvents::Break(time_interval) => {
                    break_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::Toolbox(time_interval) => {
                    toolbox_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::OffShift(time_interval) => {
                    off_shift_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::NonProductiveTime(time_interval) => {
                    non_productive_time += time_interval.duration();
                    current_time += time_interval.duration();
                    if first_fitness {
                        prev_fitness += time_interval.duration();
                    } else {
                        next_fitness += time_interval.duration();
                    }
                }
                OperationalEvents::Unavailable(_) => {
                    if !first_fitness {
                        assert!(assignment == all_events.clone().last().unwrap());
                        let marginal_fitness_time_delta = prev_fitness + next_fitness;
                        self.update_marginal_fitness(
                            current_work_order_activity
                                .expect("This will happen if there are no work orders scheduled"),
                            marginal_fitness_time_delta,
                        );
                    }
                }
            }
        }

        // assert_eq!(current_time, self.availability.end_date);
        equality_between_time_interval_and_assignments(&all_events.clone().collect::<Vec<_>>());

        assert!(is_assignments_in_bounds(
            &all_events.clone().collect(),
            &self.parameters.availability
        ));

        assert!(no_overlap(&all_events.collect::<Vec<_>>()));

        let total_time =
            wrench_time + break_time + off_shift_time + toolbox_time + non_productive_time;
        assert_eq!(total_time, self.parameters.availability.duration());

        event!(Level::TRACE, wrench_time = ?wrench_time,
        break_time = ?break_time,
        toolbox_time = ?toolbox_time,
        non_productive_time = ?non_productive_time);
        let new_objective_value = ((wrench_time).num_seconds() * 100) as u64
            / (wrench_time + break_time + toolbox_time + non_productive_time).num_seconds() as u64;

        let old_objective_value = self.solution.objective_value;

        self.solution.objective_value = new_objective_value;
        if new_objective_value > old_objective_value {
            event!(Level::INFO, operational_objective_value_better = ?new_objective_value);
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            event!(Level::INFO, operational_objective_value_worse = ?new_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        self.solution_intermediate.0.clear();
        let work_order_activities = &self
            .loaded_shared_solution
            .supervisor
            .operational_state_machine
            .iter()
            .filter(|(id_woa, del)| id_woa.0 == self.id && (del.is_assign() || del.is_assess()))
            .map(|(id_woa, _)| id_woa.1)
            .collect::<HashSet<_>>();

        for work_order_activity in work_order_activities {
            let operational_parameter = match self
                .parameters
                .work_order_parameters
                .get(work_order_activity)
            {
                Some(operational_parameter) => operational_parameter,
                None => continue,
            };

            let start_time = self
                .determine_first_available_start_time(work_order_activity, operational_parameter)
                .with_context(|| format!("{:#?}", work_order_activity))?;

            let assignments = self.determine_wrench_time_assignment(
                *work_order_activity,
                operational_parameter,
                start_time,
            );

            self.solution.try_insert(*work_order_activity, assignments);
        }

        let mut current_time = self.parameters.availability.start_date;

        // Fill the schedule
        loop {
            match self.solution.containing_operational_solution(current_time) {
                ContainOrNextOrNone::Contain(operational_solution) => {
                    current_time = operational_solution.finish_time();
                }
                ContainOrNextOrNone::Next(operational_solution) => {
                    let (new_current_time, operational_event) = self
                        .determine_next_event_non_productive(
                            &mut current_time,
                            Some(operational_solution),
                        );
                    assert!(!operational_event.is_wrench_time());
                    assert_eq!(
                        operational_event.time_delta(),
                        new_current_time - current_time
                    );
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time);
                    current_time = new_current_time;
                    self.solution_intermediate.0.push(assignment);
                }
                ContainOrNextOrNone::None => {
                    let (new_current_time, operational_event) =
                        self.determine_next_event_non_productive(&mut current_time, None);
                    assert!(!operational_event.is_wrench_time());
                    assert_eq!(
                        operational_event.time_delta(),
                        new_current_time - current_time
                    );
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time);
                    current_time = new_current_time;
                    self.solution_intermediate.0.push(assignment);
                }
            };

            if current_time >= self.parameters.availability.finish_date {
                self.solution_intermediate.0.last_mut().unwrap().finish =
                    self.parameters.availability.finish_date;
                break;
            };
        }

        Ok(())
    }

    fn unschedule(&mut self) -> Result<()> {
        let operational_solutions_len = self.solution.scheduled_work_order_activities.len();

        let operational_solutions_filtered: Vec<WorkOrderActivity> =
            self.solution.scheduled_work_order_activities[1..operational_solutions_len - 1]
                .choose_multiple(
                    &mut self.parameters.options.rng,
                    self.parameters.options.number_of_activities,
                )
                .map(|operational_solution| operational_solution.0)
                .collect();

        for operational_solution in &operational_solutions_filtered {
            self.unschedule_single_work_order_activity((
                operational_solution.0,
                operational_solution.1,
            ))
            .with_context(|| {
                format!(
                    "{:?} from {:?}\ncould not be unscheduled",
                    operational_solution, &operational_solutions_filtered
                )
            })?
        }
        Ok(())
    }
}

impl Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive> {
    fn determine_wrench_time_assignment(
        &self,
        work_order_activity: WorkOrderActivity,
        operational_parameter: &OperationalParameter,
        start_time: DateTime<Utc>,
    ) -> Vec<Assignment> {
        assert_ne!(operational_parameter.work, Work::from(0.0));
        assert!(!operational_parameter.operation_time_delta.is_zero());
        let mut assigned_work: Vec<Assignment> = vec![];
        let mut remaining_combined_work = operational_parameter.operation_time_delta;
        let mut current_time = start_time;

        while !remaining_combined_work.is_zero() {
            let next_event = self.determine_next_event(&current_time);

            if next_event.0.is_zero() {
                let finish_time = current_time + next_event.1.time_delta();
                assigned_work.push(Assignment::new(next_event.1, current_time, finish_time));
                current_time = finish_time;
            } else if next_event.0 < remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime((
                        TimeInterval::new(
                            current_time.time(),
                            (current_time + next_event.0).time(),
                        ),
                        work_order_activity,
                    )),
                    current_time,
                    current_time + next_event.0,
                ));
                current_time += next_event.0;
                remaining_combined_work -= next_event.0;
            } else if next_event.0 >= remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime((
                        TimeInterval::new(
                            current_time.time(),
                            (current_time + remaining_combined_work).time(),
                        ),
                        work_order_activity,
                    )),
                    current_time,
                    current_time + remaining_combined_work,
                ));
                current_time += next_event.0;
                remaining_combined_work = TimeDelta::zero();
            }
        }
        assert_ne!(assigned_work.len(), 0);
        assigned_work
    }

    fn unschedule_single_work_order_activity(
        &mut self,
        work_order_and_activity_number: WorkOrderActivity,
    ) -> Result<()> {
        ensure!(self
            .solution
            .scheduled_work_order_activities
            .iter()
            .any(|os| os.0 == work_order_and_activity_number));
        dbg!(&self.solution.scheduled_work_order_activities.len());

        self.solution
            .scheduled_work_order_activities
            .retain(|os| os.0 != work_order_and_activity_number);
        dbg!(&self.solution.scheduled_work_order_activities.len());

        ensure!(!self
            .solution
            .scheduled_work_order_activities
            .iter()
            .any(|os| os.0 == work_order_and_activity_number));
        Ok(())
    }

    fn determine_next_event(&self, current_time: &DateTime<Utc>) -> (TimeDelta, OperationalEvents) {
        let break_diff = (
            self.parameters.break_interval.start - current_time.time(),
            OperationalEvents::Break(self.parameters.break_interval.clone()),
        );

        let toolbox_diff = (
            self.parameters.toolbox_interval.start - current_time.time(),
            OperationalEvents::Toolbox(self.parameters.toolbox_interval.clone()),
        );

        let off_shift_diff = (
            self.parameters.off_shift_interval.start - current_time.time(),
            OperationalEvents::OffShift(self.parameters.off_shift_interval.clone()),
        );

        [break_diff, toolbox_diff, off_shift_diff]
            .iter()
            .filter(|&diff_event| diff_event.0.num_seconds() >= 0)
            .min_by_key(|&diff_event| diff_event.0.num_seconds())
            .cloned()
            .unwrap()
    }

    fn determine_first_available_start_time(
        &self,
        work_order_activity: &WorkOrderActivity,
        operational_parameter: &OperationalParameter,
    ) -> Result<DateTime<Utc>> {
        // Here we load in the `TacticalSolution` from the `loaded_shared_solution`. This should function to
        // make the code work more seamlessly with the `TacticalSolution`. Are we doing this correctly? I do
        // not think that we are. The thing is that the supervisor can force a work order here and then the
        // Tactical and Strategic Agent has to respect that. That means that initialially this could be
        // None, but we should strive to make this as perfect as possible. If there is a tactical days we should
        // use that. If there is a Strategic period we should use that. If there is none we should check the
        // manual part. The issue here is not that it is not scheduled, the issue is that the entry does not
        // exist. What should you do here?
        let tactical_days_option = self
            .loaded_shared_solution
            .tactical
            .tactical_work_orders
            .0
            .get(&work_order_activity.0);

        // .expect("This should always be present. If this occurs you should check the initialization. The implementation is that the tactical and strategic algorithm always provide a key for each WorkOrderNumber");

        let strategic_period_option = self
            .loaded_shared_solution
            .strategic
            .strategic_scheduled_work_orders
            .get(&work_order_activity.0);

        // .expect("This should always be present. If this occurs you should check the initialization. The implementation is that the tactical and strategic algorithm always provide a key for each WorkOrderNumber");

        let (start_window, end_window) = match (strategic_period_option, tactical_days_option) {
            // What is actually happening here?
            (None, None) => (
                &self.parameters.availability.start_date,
                &self.parameters.availability.finish_date,
            ),
            (_, Some(WhereIsWorkOrder::Tactical(activities))) => {
                let scheduled_days = &activities.0.get(&work_order_activity.1).unwrap().scheduled;

                let start = scheduled_days.first().unwrap().0.date();
                let end = scheduled_days.last().unwrap().0.date();

                (start, end)
            }
            (Some(Some(period)), _) => (period.start_date(), period.end_date()),

            _ => bail!(
                "{}: {:#?}\n{}: {:#?}\n",
                std::any::type_name::<StrategicSolution>(),
                strategic_period_option,
                std::any::type_name::<TacticalSolution>(),
                tactical_days_option
            ),
        };

        for operational_solution in self.solution.scheduled_work_order_activities.windows(2) {
            let start_of_availability = {
                let mut current_time = operational_solution[0].1.assignments.last().unwrap().finish;

                if current_time < *start_window {
                    current_time = *start_window;
                }

                let current_time_option = self.update_current_time_based_on_event(current_time);

                current_time = match current_time_option {
                    Some(new_current_time) => new_current_time,
                    None => current_time,
                };

                loop {
                    let (time_to_next_event, next_event) = self.determine_next_event(&current_time);

                    if time_to_next_event.is_zero() {
                        current_time += next_event.time_delta();
                    } else {
                        break current_time;
                    }
                }
            };

            let end_of_availability = operational_solution[1].1.assignments.first().unwrap().start;

            if (*end_window).min(end_of_availability) - (*start_window).max(start_of_availability)
                > operational_parameter.operation_time_delta
            {
                return Ok(*start_window.max(&start_of_availability));
            }
        }

        let mut current_time = *start_window;

        let current_time_option = self.update_current_time_based_on_event(current_time);

        current_time = match current_time_option {
            Some(new_current_time) => new_current_time,
            None => current_time,
        };

        loop {
            let (time_to_next_event, next_event) = self.determine_next_event(&current_time);

            if time_to_next_event.is_zero() {
                current_time += next_event.time_delta();
            } else {
                break Ok(current_time);
            }
        }
    }

    fn update_current_time_based_on_event(
        &self,
        mut current_time: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.parameters.off_shift_interval.contains(&current_time) {
            let off_shift_interval_end = self.parameters.off_shift_interval.end;
            if off_shift_interval_end < current_time.time() {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.parameters.break_interval.contains(&current_time) {
            let break_interval_end = self.parameters.break_interval.end;
            if break_interval_end < current_time.time() {
                current_time = current_time.with_time(break_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(break_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.parameters.toolbox_interval.contains(&current_time) {
            let toolbox_interval_end = self.parameters.toolbox_interval.end;
            if toolbox_interval_end < current_time.time() {
                current_time = current_time.with_time(toolbox_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(toolbox_interval_end).unwrap();
                Some(current_time)
            }
        } else {
            None
        }
    }
}

fn no_overlap(events: &Vec<Assignment>) -> bool {
    for event_1 in events {
        for event_2 in events {
            if event_1 == event_2 {
                continue;
            }

            if (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start) {
                continue;
            } else {
                dbg!(event_1);
                dbg!(event_2);
                return false;
            }
        }
    }
    true
}

fn no_overlap_by_ref(events: Vec<&Assignment>) -> bool {
    for event_1 in &events {
        for event_2 in &events {
            if event_1 == event_2 {
                continue;
            }

            if (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start) {
                continue;
            } else {
                dbg!(event_1);
                dbg!(event_2);
                return false;
            }
        }
    }
    true
}

fn is_assignments_in_bounds(events: &Vec<Assignment>, availability: &Availability) -> bool {
    for event in events {
        if event.start < availability.start_date && !event.event_type.unavail() {
            dbg!(event, availability);
            return false;
        }
        if availability.finish_date < event.finish && !event.event_type.unavail() {
            dbg!(event, availability);
            return false;
        }
    }
    true
}

fn equality_between_time_interval_and_assignments(all_events: &Vec<Assignment>) {
    for assignment in all_events {
        assert_eq!(assignment.start.time(), assignment.event_type.start_time());
        assert_eq!(
            assignment.finish.time(),
            assignment.event_type.finish_time()
        );
        assert_eq!(
            assignment.event_type.time_delta(),
            assignment.finish - assignment.start
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use chrono::{DateTime, NaiveTime, TimeDelta, Utc};
    use proptest::prelude::*;
    use shared_types::{
        agents::operational::{OperationalConfiguration, TimeInterval},
        scheduling_environment::{
            time_environment::period::Period,
            work_order::{
                operation::{ActivityNumber, Work},
                WorkOrderNumber,
            },
            worker_environment::{availability::Availability, resources::Id},
            SchedulingEnvironment,
        },
        OperationalConfigurationAll,
    };

    use crate::agents::{
        operational_agent::{
            algorithm::{operational_parameter::OperationalParameters, OperationalEvents},
            OperationalOptions,
        },
        traits::Parameters,
        Algorithm, AlgorithmUtils, ArcSwapSharedSolution, OperationalSolution, Solution,
        WhereIsWorkOrder,
    };
    use anyhow::Result;

    use super::OperationalParameter;

    #[test]
    fn test_determine_next_event_1() -> Result<()> {
        let availability_start: DateTime<Utc> =
            DateTime::parse_from_rfc3339("2024-05-16T07:00:00Z")
                .unwrap()
                .to_utc();
        let availability_end: DateTime<Utc> = DateTime::parse_from_rfc3339("2024-05-30T15:00:00Z")
            .unwrap()
            .to_utc();

        let break_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let off_shift_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
        let toolbox_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        );

        // An OperationalAgent should be be able to run without this. It is crucial that
        // it is functioning correctly.

        let operational_configuration = OperationalConfiguration::new(
            Availability::new(availability_start, availability_end),
            break_interval,
            off_shift_interval.clone(),
            toolbox_interval,
        );

        let mut scheduling_environment = SchedulingEnvironment::default();

        let id = &Id::new("TEST_OPERATIONAL", vec![], vec![]);

        let operational_configuration_all =
            OperationalConfigurationAll::new(id.clone(), 6.0, operational_configuration);

        scheduling_environment
            .worker_environment
            .agent_environment
            .operational
            .insert(id.clone(), operational_configuration_all);

        let scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

        let operational_parameters = OperationalParameters::new(
            id,
            OperationalOptions::default(),
            &scheduling_environment.lock().unwrap(),
        )?;

        let operational_solution = OperationalSolution::new(&operational_parameters);

        let operational_algorithm = Algorithm::new(
            id,
            operational_solution,
            operational_parameters,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T12:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());

        assert_eq!(next_event, OperationalEvents::OffShift(off_shift_interval));
        Ok(())
    }

    #[test]
    fn test_determine_next_event_2() -> Result<()> {
        let availability_start: DateTime<Utc> =
            DateTime::parse_from_rfc3339("2024-05-16T07:00:00Z")
                .unwrap()
                .to_utc();
        let availability_end: DateTime<Utc> = DateTime::parse_from_rfc3339("2024-05-30T15:00:00Z")
            .unwrap()
            .to_utc();

        let break_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let off_shift_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
        let toolbox_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        );

        let operational_configuration = OperationalConfiguration::new(
            Availability::new(availability_start, availability_end),
            break_interval.clone(),
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let mut scheduling_environment = SchedulingEnvironment::default();

        let id = &Id::new("TEST_OPERATIONAL", vec![], vec![]);

        let operational_configuration_all =
            OperationalConfigurationAll::new(id.clone(), 6.0, operational_configuration);

        scheduling_environment
            .worker_environment
            .agent_environment
            .operational
            .insert(id.clone(), operational_configuration_all);

        let scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

        let operational_parameters = OperationalParameters::new(
            &id,
            OperationalOptions::default(),
            &scheduling_environment.lock().unwrap(),
        )?;

        let operational_solution = OperationalSolution::new(&operational_parameters);

        let operational_algorithm = Algorithm::new(
            &Id::new("TEST_OPERATIONAL", vec![], vec![]),
            operational_solution,
            operational_parameters,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T00:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());
        assert_eq!(next_event, OperationalEvents::Toolbox(toolbox_interval));
        Ok(())
    }

    #[test]
    fn test_determine_next_event_3() -> Result<()> {
        let availability_start: DateTime<Utc> =
            DateTime::parse_from_rfc3339("2024-05-16T07:00:00Z")
                .unwrap()
                .to_utc();
        let availability_end: DateTime<Utc> = DateTime::parse_from_rfc3339("2024-05-30T15:00:00Z")
            .unwrap()
            .to_utc();

        let break_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let off_shift_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
        let toolbox_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        );
        let operational_configuration = OperationalConfiguration::new(
            Availability::new(availability_start, availability_end),
            break_interval.clone(),
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let mut scheduling_environment = SchedulingEnvironment::default();

        let id = &Id::new("TEST_OPERATIONAL", vec![], vec![]);

        let operational_configuration_all =
            OperationalConfigurationAll::new(id.clone(), 6.0, operational_configuration);

        scheduling_environment
            .worker_environment
            .agent_environment
            .operational
            .insert(id.clone(), operational_configuration_all);

        let scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

        let operational_parameters = OperationalParameters::new(
            &id,
            OperationalOptions::default(),
            &scheduling_environment.lock().unwrap(),
        )?;

        let operational_solution = OperationalSolution::new(&operational_parameters);

        let operational_algorithm = Algorithm::new(
            &Id::new("TEST_OPERATIONAL", vec![], vec![]),
            operational_solution,
            operational_parameters,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T01:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 6, 0).unwrap());
        assert_eq!(next_event, OperationalEvents::Toolbox(toolbox_interval));
        Ok(())
    }

    #[test]
    fn test_determine_first_available_start_time() -> Result<()> {
        let availability_start: DateTime<Utc> =
            DateTime::parse_from_rfc3339("2024-10-07T07:00:00Z")
                .unwrap()
                .to_utc();
        let availability_end: DateTime<Utc> = DateTime::parse_from_rfc3339("2024-10-20T15:00:00Z")
            .unwrap()
            .to_utc();

        let break_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let off_shift_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
        let toolbox_interval = TimeInterval::new(
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        );
        let operational_configuration = OperationalConfiguration::new(
            Availability::new(availability_start, availability_end),
            break_interval.clone(),
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let mut scheduling_environment = SchedulingEnvironment::default();

        let id = &Id::new("TEST_OPERATIONAL", vec![], vec![]);

        let operational_configuration_all =
            OperationalConfigurationAll::new(id.clone(), 6.0, operational_configuration);

        scheduling_environment
            .worker_environment
            .agent_environment
            .operational
            .insert(id.clone(), operational_configuration_all);

        let scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

        let operational_parameters = OperationalParameters::new(
            &id,
            OperationalOptions::default(),
            &scheduling_environment.lock().unwrap(),
        )?;

        let operational_solution = OperationalSolution::new(&operational_parameters);

        let mut operational_algorithm = Algorithm::new(
            &Id::new("TEST_OPERATIONAL", vec![], vec![]),
            operational_solution,
            operational_parameters,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        operational_algorithm.load_shared_solution();

        let mut strategic_updated_shared_solution =
            (**operational_algorithm.loaded_shared_solution).clone();

        strategic_updated_shared_solution
            .strategic
            .strategic_scheduled_work_orders
            .insert(
                WorkOrderNumber(0),
                Some(Period::from_str("2024-W41-42").unwrap()),
            );

        operational_algorithm
            .arc_swap_shared_solution
            .0
            .store(Arc::new(strategic_updated_shared_solution));

        operational_algorithm.load_shared_solution();
        let mut tactical_updated_shared_solution =
            (**operational_algorithm.loaded_shared_solution).clone();

        tactical_updated_shared_solution
            .tactical
            .tactical_work_orders
            .0
            .insert(WorkOrderNumber(0), WhereIsWorkOrder::NotScheduled);

        operational_algorithm
            .arc_swap_shared_solution
            .0
            .store(Arc::new(tactical_updated_shared_solution));

        operational_algorithm.load_shared_solution();

        let operational_parameter = OperationalParameter::new(Work::from(20.0), Work::from(0.0))
            .expect("Work has to be non-zero to create an OperationalParameter");

        let start_time = operational_algorithm
            .determine_first_available_start_time(&(WorkOrderNumber(0), 0), &operational_parameter)
            .unwrap();

        assert_eq!(
            start_time,
            DateTime::parse_from_rfc3339("2024-10-07T08:00:00Z")
                .unwrap()
                .to_utc()
        );
        Ok(())
    }

    fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    proptest! {
        #[test]
        fn test_reverse(s in ".*") {
            let reversed = reverse(&s);
            // Check that reversing twice yields the original string
            prop_assert_eq!(s, reverse(&reversed));
        }
    }

    proptest! {
        #[test]
        fn test_with_custom_strategy(vec in prop::collection::vec(0..100i32, 0..100)) {
            let reversed: Vec<i32> = vec.iter().rev().cloned().collect();
            let double_reversed: Vec<i32> = reversed.iter().rev().cloned().collect();

            prop_assert_eq!(vec, double_reversed);
        }
    }
}
