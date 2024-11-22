pub mod operational_events;
pub mod operational_parameter;
pub mod operational_solution;

use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use arc_swap::Guard;
use chrono::{DateTime, TimeDelta, Utc};
use itertools::Itertools;
use operational_events::OperationalEvents;
use operational_parameter::{OperationalParameter, OperationalParameters};
use operational_solution::{Assignment, OperationalAssignment, OperationalFunctions};
use rand::seq::SliceRandom;
use shared_types::{
    operational::{
        operational_request_resource::OperationalResourceRequest,
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_request_time::OperationalTimeRequest,
        operational_response_resource::OperationalResourceResponse,
        operational_response_scheduling::OperationalSchedulingResponse,
        operational_response_time::OperationalTimeResponse, TimeInterval,
    },
    scheduling_environment::{
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::{availability::Availability, resources::Id},
    },
};
use tracing::{event, Level};

use crate::agents::{
    traits::LargeNeighborHoodSearch, ArcSwapSharedSolution, OperationalSolution, SharedSolution,
};

use super::OperationalConfiguration;

pub type OperationalObjectiveValue = u64;

pub struct OperationalAlgorithm {
    pub operational_solution: OperationalSolution,
    pub operational_non_productive: OperationalNonProductive,
    pub operational_parameters: OperationalParameters,
    pub history_of_dropped_operational_parameters: HashSet<WorkOrderActivity>,
    pub arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

#[derive(Clone)]
pub struct OperationalNonProductive(Vec<Assignment>);

impl OperationalAlgorithm {
    pub fn new(
        operational_configuration: OperationalConfiguration,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    ) -> Self {
        let loaded_shared_solution = arc_swap_shared_solution.0.load();
        Self {
            operational_solution: OperationalSolution::new(Vec::new()),
            operational_non_productive: OperationalNonProductive(Vec::new()),
            operational_parameters: OperationalParameters::default(),
            history_of_dropped_operational_parameters: HashSet::new(),
            availability: operational_configuration.availability,
            off_shift_interval: operational_configuration.off_shift_interval,
            break_interval: operational_configuration.break_interval,
            toolbox_interval: operational_configuration.toolbox_interval,
            arc_swap_shared_solution,
            loaded_shared_solution,
        }
    }

    fn remove_drop_delegates(&mut self, operational_agent: &Id) -> HashSet<WorkOrderActivity> {
        let mut removed_work_order_activities = HashSet::new();

        let mut operational_shared_solution = self
            .loaded_shared_solution
            .supervisor
            .state_of_agent(operational_agent)
            .into_keys();

        self.operational_parameters
            .work_order_parameters
            .retain(|woa, _| {
                if operational_shared_solution.contains(woa) {
                    removed_work_order_activities.insert(woa.clone());
                    true
                } else {
                    false
                }
            });
        removed_work_order_activities
    }

    pub fn remove_delegate_drop(&mut self, operational_agent: &Id) {
        let woas_to_be_deleted = self.remove_drop_delegates(operational_agent);

        for work_order_activity in woas_to_be_deleted {
            self.operational_solution
                .work_order_activities
                .retain(|os| os.0 != work_order_activity)
        }
    }

    pub fn insert_operational_parameter(
        &mut self,
        work_order_activity: WorkOrderActivity,
        operational_parameters: OperationalParameter,
    ) -> Option<OperationalParameter> {
        self.operational_parameters
            .work_order_parameters
            .insert(work_order_activity, operational_parameters)
    }

    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<OperationalAssignment>,
    ) -> (DateTime<Utc>, OperationalEvents) {
        if self.break_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.break_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (new_current_time, OperationalEvents::Break(time_interval))
        } else if self.off_shift_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.off_shift_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::OffShift(self.off_shift_interval.clone()),
            )
        } else if self.toolbox_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.toolbox_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::Toolbox(self.toolbox_interval.clone()),
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
                if self.availability.end_date < new_current_time {
                    new_current_time = self.availability.end_date;
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

    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    ) {
        let time_delta_usize = time_delta.num_seconds() as u64;

        self.operational_solution
            .work_order_activities
            .iter_mut()
            .find(|oper_sol| oper_sol.0 == work_order_activity_previous)
            .unwrap()
            .1
            .marginal_fitness
            .0 = time_delta_usize;
    }

    pub(crate) fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub(crate) fn make_atomic_pointer_swap(&self, operational_id: &Id) {
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
                .insert(operational_id.clone(), self.operational_solution.clone());
            Arc::new(shared_solution)
        });
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

impl LargeNeighborHoodSearch for OperationalAlgorithm {
    type BetterSolution = bool;
    type SchedulingRequest = OperationalSchedulingRequest;

    type SchedulingResponse = OperationalSchedulingResponse;

    type ResourceRequest = OperationalResourceRequest;

    type ResourceResponse = OperationalResourceResponse;

    type TimeRequest = OperationalTimeRequest;

    type TimeResponse = OperationalTimeResponse;

    type SchedulingUnit = (WorkOrderNumber, ActivityNumber);

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        // Here we should determine the objective based on the highest needed skill. Meaning that a MTN-TURB should not bid highly
        // on a MTN-MECH job. I think that this will be very interesting to solve.
        let operational_events: Vec<Assignment> = self
            .operational_solution
            .work_order_activities
            .iter()
            .flat_map(|(_, os)| os.assignments.iter())
            .cloned()
            .collect();

        event!(Level::DEBUG, operational_events_len = ?operational_events.iter().filter(|val| val.event_type.is_wrench_time()).collect::<Vec<_>>().len());

        let all_events = operational_events
            .into_iter()
            .chain(self.operational_non_productive.0.clone())
            .sorted_unstable_by_key(|ass| ass.start);

        let mut current_time = self.availability.start_date;

        let mut wrench_time: TimeDelta = TimeDelta::zero();
        let mut break_time: TimeDelta = TimeDelta::zero();
        let mut off_shift_time: TimeDelta = TimeDelta::zero();
        let mut toolbox_time: TimeDelta = TimeDelta::zero();
        let mut non_productive_time: TimeDelta = TimeDelta::zero();

        let mut prev_fitness: TimeDelta = TimeDelta::zero();
        let mut next_fitness: TimeDelta = TimeDelta::zero();
        let mut first_fitness: bool = true;
        let mut current_work_order_activity: Option<WorkOrderActivity> = None;
        event!(Level::ERROR, operational_event = ?all_events);
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
                OperationalEvents::Unavailable(_) => (),
            }
        }

        // assert_eq!(current_time, self.availability.end_date);

        equality_between_time_interval_and_assignments(&all_events.clone().collect::<Vec<_>>());

        assert!(is_assignments_in_bounds(
            &all_events.clone().collect(),
            &self.availability
        ));

        assert!(no_overlap(&all_events.collect::<Vec<_>>()));

        let total_time =
            wrench_time + break_time + off_shift_time + toolbox_time + non_productive_time;
        assert_eq!(total_time, self.availability.duration());

        event!(Level::TRACE, wrench_time = ?wrench_time,
        break_time = ?break_time,
        toolbox_time = ?toolbox_time,
        non_productive_time = ?non_productive_time);
        let new_value = ((wrench_time).num_seconds() * 100) as u64
            / (wrench_time + break_time + toolbox_time + non_productive_time).num_seconds() as u64;

        let old_value = self.operational_solution.objective_value;

        if new_value > old_value {
            self.operational_solution.objective_value = new_value;
            true
        } else {
            false
        }
    }

    fn schedule(&mut self) -> Result<()> {
        self.operational_non_productive.0.clear();
        for (work_order_activity, operational_parameter) in
            &self.operational_parameters.work_order_parameters
        {
            let start_time = self
                .determine_first_available_start_time(work_order_activity, operational_parameter);

            let assignments = self.determine_wrench_time_assignment(
                *work_order_activity,
                operational_parameter,
                start_time,
            );

            self.operational_solution
                .try_insert(*work_order_activity, assignments);

            event!(Level::TRACE, number_of_operations = ?self.operational_solution.work_order_activities.len());
        }

        // fill the schedule
        let mut current_time = self.availability.start_date;

        loop {
            match self
                .operational_solution
                .containing_operational_solution(current_time)
            {
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
                    self.operational_non_productive.0.push(assignment);
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
                    self.operational_non_productive.0.push(assignment);
                }
            };

            if current_time >= self.availability.end_date {
                self.operational_non_productive.0.last_mut().unwrap().finish =
                    self.availability.end_date;
                break;
            };
        }
        Ok(())
    }

    fn unschedule(&mut self, work_order_and_activity_number: Self::SchedulingUnit) -> Result<()> {
        let unscheduled_operational_solution = self
            .operational_solution
            .work_order_activities
            .iter()
            .find(|operational_solution| operational_solution.0 == work_order_and_activity_number)
            .take();
        unscheduled_operational_solution.expect("There was nothing in the operational solution");
        Ok(())
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse> {
        todo!()
    }

    fn update_time_state(&mut self, _message: Self::TimeRequest) -> Result<Self::TimeResponse> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        _message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> {
        todo!();
    }
}

impl OperationalAlgorithm {
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

    fn determine_next_event(&self, current_time: &DateTime<Utc>) -> (TimeDelta, OperationalEvents) {
        let break_diff = (
            self.break_interval.start - current_time.time(),
            OperationalEvents::Break(self.break_interval.clone()),
        );

        let toolbox_diff = (
            self.toolbox_interval.start - current_time.time(),
            OperationalEvents::Toolbox(self.toolbox_interval.clone()),
        );

        let off_shift_diff = (
            self.off_shift_interval.start - current_time.time(),
            OperationalEvents::OffShift(self.off_shift_interval.clone()),
        );

        [break_diff, toolbox_diff, off_shift_diff]
            .iter()
            .filter(|&diff_event| diff_event.0.num_seconds() >= 0)
            .min_by_key(|&diff_event| diff_event.0.num_seconds())
            .cloned()
            .unwrap()
    }

    pub fn unschedule_random_work_order_activies(
        &mut self,
        rng: &mut impl rand::Rng,
        number_of_activities: usize,
    ) {
        let operational_solutions: Vec<WorkOrderActivity> = self
            .operational_solution
            .work_order_activities
            .choose_multiple(rng, number_of_activities)
            .map(|operational_solution| operational_solution.0)
            .collect();

        for operational_solution in operational_solutions {
            self.unschedule((operational_solution.0, operational_solution.1))
                .expect("OperationalAgent could not unschedule correctly");
        }
    }

    fn determine_first_available_start_time(
        &self,
        work_order_activity: &WorkOrderActivity,
        operational_parameter: &OperationalParameter,
    ) -> DateTime<Utc> {
        // What should be done here? I think that the goal is to create a
        let tactical_days_option = self
            .loaded_shared_solution
            .tactical
            .tactical_days
            .get(&work_order_activity.0)
            .expect("This should always be present. If this occurs you should check the initialization. The implementation is that the tactical and strategic algorithm always provide a key for each WorkOrderNumber");

        let strategic_period_option = self
            .loaded_shared_solution
            .strategic
            .strategic_periods
            .get(&work_order_activity.0)
            .expect("This should always be present. If this occurs you should check the initialization. The implementation is that the tactical and strategic algorithm always provide a key for each WorkOrderNumber");

        let (start_window, end_window) = match (strategic_period_option, tactical_days_option) {
            (Some(_period), Some(activities)) => {
                let scheduled_days = &activities.get(&work_order_activity.1).unwrap().scheduled;

                let start = scheduled_days.first().unwrap().0.date();
                let end = scheduled_days.last().unwrap().0.date();

                (start, end)
            }
            (Some(period), None) => (period.start_date(), period.end_date()),

            (None, Some(_)) => todo!(),
            // (&self.availability.start_date, &self.availability.end_date)
            (None, None) => panic!("This should not happen yet, either the tactical xor the strategic has a solution available"),
        };
        for operational_solution in self.operational_solution.work_order_activities.windows(2) {
            let start_of_interval = {
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

            let end_of_interval = operational_solution[1].1.assignments.first().unwrap().start;

            if (*end_window).min(end_of_interval) - (*start_window).max(start_of_interval)
                > operational_parameter.operation_time_delta
            {
                return *start_window.max(&start_of_interval);
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
                break current_time;
            }
        }
    }

    fn update_current_time_based_on_event(
        &self,
        mut current_time: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.off_shift_interval.contains(&current_time) {
            let off_shift_interval_end = self.off_shift_interval.end;
            if off_shift_interval_end < current_time.time() {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.break_interval.contains(&current_time) {
            let break_interval_end = self.break_interval.end;
            if break_interval_end < current_time.time() {
                current_time = current_time.with_time(break_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(break_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.toolbox_interval.contains(&current_time) {
            let toolbox_interval_end = self.toolbox_interval.end;
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
        if availability.end_date < event.finish && !event.event_type.unavail() {
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
    use std::{str::FromStr, sync::Arc};

    use chrono::{DateTime, NaiveTime, TimeDelta, Utc};
    use proptest::prelude::*;
    use shared_types::{
        operational::{OperationalConfiguration, TimeInterval},
        scheduling_environment::{
            time_environment::period::Period,
            work_order::{
                operation::{ActivityNumber, Work},
                WorkOrderNumber,
            },
            worker_environment::availability::Availability,
        },
    };

    use crate::agents::{operational_agent::algorithm::OperationalEvents, ArcSwapSharedSolution};

    use super::{OperationalAlgorithm, OperationalParameter};

    #[test]
    fn test_determine_next_event_1() {
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
            break_interval,
            off_shift_interval.clone(),
            toolbox_interval,
        );

        let operational_algorithm = OperationalAlgorithm::new(
            operational_configuration,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T12:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());

        assert_eq!(next_event, OperationalEvents::OffShift(off_shift_interval));
    }

    #[test]
    fn test_determine_next_event_2() {
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
            break_interval,
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let operational_algorithm = OperationalAlgorithm::new(
            operational_configuration,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T00:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());
        assert_eq!(next_event, OperationalEvents::Toolbox(toolbox_interval));
    }

    #[test]
    fn test_determine_next_event_3() {
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
            break_interval,
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let operational_algorithm = OperationalAlgorithm::new(
            operational_configuration,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        let current_time = DateTime::parse_from_rfc3339("2024-05-20T01:00:00Z")
            .unwrap()
            .to_utc();

        let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

        assert_eq!(time_delta, TimeDelta::new(3600 * 6, 0).unwrap());
        assert_eq!(next_event, OperationalEvents::Toolbox(toolbox_interval));
    }

    #[test]
    fn test_determine_first_available_start_time() {
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
            break_interval,
            off_shift_interval.clone(),
            toolbox_interval.clone(),
        );

        let mut operational_algorithm = OperationalAlgorithm::new(
            operational_configuration,
            Arc::new(ArcSwapSharedSolution::default()),
        );

        operational_algorithm.load_shared_solution();

        let mut strategic_updated_shared_solution =
            (**operational_algorithm.loaded_shared_solution).clone();

        strategic_updated_shared_solution
            .strategic
            .strategic_periods
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
            .tactical_days
            .insert(WorkOrderNumber(0), None);

        operational_algorithm
            .arc_swap_shared_solution
            .0
            .store(Arc::new(tactical_updated_shared_solution));

        operational_algorithm.load_shared_solution();

        let operational_parameter = OperationalParameter::new(Work::from(20.0), Work::from(0.0));

        let start_time = operational_algorithm.determine_first_available_start_time(
            &(WorkOrderNumber(0), ActivityNumber(0)),
            &operational_parameter,
        );

        assert_eq!(
            start_time,
            DateTime::parse_from_rfc3339("2024-10-07T08:00:00Z")
                .unwrap()
                .to_utc()
        );
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
