use std::collections::HashMap;

use chrono::{DateTime, NaiveTime, TimeDelta, Utc};
use rand::seq::SliceRandom;
use shared_messages::{
    agent_error::AgentError,
    models::{
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::availability::Availability,
    },
    operational::{
        operational_request_resource::OperationalResourceRequest,
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_request_time::OperationalTimeRequest,
        operational_response_resource::OperationalResourceResponse,
        operational_response_scheduling::OperationalSchedulingResponse,
        operational_response_time::OperationalTimeResponse, TimeInterval,
    },
};

use crate::agents::traits::LargeNeighborHoodSearch;

use super::{Assigned, OperationalConfiguration};

pub type OperationalObjective = f64;

#[derive(Clone)]
pub struct OperationalAlgorithm {
    pub objective_value: OperationalObjective,
    pub operational_solutions: OperationalSolutions,
    pub operational_non_productive: OperationalNonProductive,
    pub operational_parameters: HashMap<(WorkOrderNumber, ActivityNumber), OperationalParameter>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

#[derive(Clone)]
pub struct OperationalNonProductive(Vec<Assignment>);

#[allow(dead_code)]
impl OperationalAlgorithm {
    pub fn new(operational_configuration: OperationalConfiguration) -> Self {
        Self {
            objective_value: f64::INFINITY,
            operational_solutions: OperationalSolutions(Vec::new()),
            operational_non_productive: OperationalNonProductive(Vec::new()),
            operational_parameters: HashMap::new(),
            availability: operational_configuration.availability,
            off_shift_interval: operational_configuration.off_shift_interval,
            break_interval: operational_configuration.break_interval,
            toolbox_interval: operational_configuration.toolbox_interval,
        }
    }

    #[allow(dead_code)]
    pub fn insert_optimized_operation(
        &mut self,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        operational_parameters: OperationalParameter,
    ) {
        self.operational_parameters
            .insert((work_order_number, activity_number), operational_parameters);
    }

    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<OperationalSolution>,
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
        next_operation: Option<OperationalSolution>,
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
}

#[derive(Clone)]
pub struct OperationalSolutions(
    pub Vec<(WorkOrderNumber, ActivityNumber, Option<OperationalSolution>)>,
);

trait OperationalFunctions {
    type Key;
    type Sequence;

    fn try_insert(&mut self, key: Self::Key, sequence: Self::Sequence);

    fn containing_operational_solution(&self, time: DateTime<Utc>) -> ContainOrNextOrNone;
}

impl OperationalFunctions for OperationalSolutions {
    type Key = (WorkOrderNumber, ActivityNumber);
    type Sequence = Vec<Assignment>;

    fn try_insert(&mut self, key: Self::Key, assignments: Self::Sequence) {
        for (index, operation_solution) in self.0.windows(2).enumerate() {
            let finish_time_solution_window = match &operation_solution[0].2 {
                Some(operational_solution) => operational_solution.finish_time(),
                None => break,
            };

            let start_time_solution_window = match &operation_solution[1].2 {
                Some(operational_solution) => operational_solution.start_time(),
                None => break,
            };

            if finish_time_solution_window < assignments.first().unwrap().start
                && assignments.last().unwrap().finish < start_time_solution_window
            {
                let operational_solution = OperationalSolution {
                    assigned: false,
                    assignments,
                };

                if self.is_operational_solution_unique(key) {
                    self.0
                        .insert(index + 1, (key.0, key.1, Some(operational_solution)));
                }
                break;
            }
        }
        if self.is_operational_solution_unique(key) {
            self.0.push((key.0, key.1, None));
        };
    }

    fn containing_operational_solution(&self, time: DateTime<Utc>) -> ContainOrNextOrNone {
        let containing: Option<OperationalSolution> = self
            .0
            .iter()
            .find(|operational_solution| operational_solution.2.as_ref().unwrap().contains(time))
            .map(|os| os.2.clone().unwrap());

        match containing {
            Some(containing) => ContainOrNextOrNone::Contain(containing),
            None => {
                let next: Option<OperationalSolution> = self
                    .0
                    .iter()
                    .find(|os| os.2.as_ref().unwrap().start_time() > time)
                    .map(|os| os.2.clone().unwrap());
                match next {
                    Some(operational_solution) => ContainOrNextOrNone::Next(operational_solution),
                    None => ContainOrNextOrNone::None,
                }
            }
        }
    }
}

impl OperationalSolutions {
    fn is_operational_solution_unique(&self, key: (WorkOrderNumber, ActivityNumber)) -> bool {
        self.0
            .iter()
            .any(|(work_order_number, activity_number, _)| {
                *work_order_number == key.0 && *activity_number == key.1
            })
    }
}

enum ContainOrNextOrNone {
    Contain(OperationalSolution),
    Next(OperationalSolution),
    None,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct OperationalSolution {
    assigned: Assigned,
    assignments: Vec<Assignment>,
}

impl OperationalSolution {
    pub fn start_time(&self) -> DateTime<Utc> {
        self.assignments.first().unwrap().start
    }

    pub fn finish_time(&self) -> DateTime<Utc> {
        self.assignments.last().unwrap().finish
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        if self.start_time() <= time && time <= self.finish_time() {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Assignment {
    pub event_type: OperationalEvents,
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

impl Assignment {
    pub fn new(event_type: OperationalEvents, start: DateTime<Utc>, finish: DateTime<Utc>) -> Self {
        assert_eq!(event_type.time_delta(), finish - start);
        assert!(start < finish);
        assert_eq!(event_type.start_time(), start.time());
        assert_eq!(event_type.finish_time(), finish.time());
        Self {
            event_type,
            start,
            finish,
        }
    }
}

impl OperationalSolution {
    pub fn new(assigned: Assigned, assignments: Vec<Assignment>) -> Self {
        Self {
            assigned,
            assignments,
        }
    }
}

#[derive(Clone)]
pub struct OperationalParameter {
    work: f64,
    preparation: f64,
    operation_time_delta: TimeDelta,
    start_window: DateTime<Utc>,
    end_window: DateTime<Utc>,
}

impl OperationalParameter {
    pub fn new(
        work: f64,
        preparation: f64,
        start_window: DateTime<Utc>,
        end_window: DateTime<Utc>,
    ) -> Self {
        let combined_time = 3600.0 * (work + preparation);
        let seconds_time = combined_time.trunc() as i64;
        let nano_time = combined_time.fract() as u32;
        let operation_time_delta = TimeDelta::new(seconds_time, nano_time).unwrap();
        Self {
            work,
            preparation,
            operation_time_delta,
            start_window,
            end_window,
        }
    }
}

impl LargeNeighborHoodSearch for OperationalAlgorithm {
    type SchedulingRequest = OperationalSchedulingRequest;

    type SchedulingResponse = OperationalSchedulingResponse;

    type ResourceRequest = OperationalResourceRequest;

    type ResourceResponse = OperationalResourceResponse;

    type TimeRequest = OperationalTimeRequest;

    type TimeResponse = OperationalTimeResponse;

    type SchedulingUnit = (WorkOrderNumber, ActivityNumber);

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        // Here we should determine the objective based on the highest needed skill. Meaning that a MTN-TURB should not bid highly
        // on a MTN-MECH job. I think that this will be very interesting to solve.
        let operational_events: Vec<Assignment> = self
            .operational_solutions
            .0
            .iter()
            .flat_map(|inner| inner.2.as_ref().unwrap().assignments.iter())
            .cloned()
            .collect();

        let mut current_time = self.availability.start_date;

        let mut wrench_time: TimeDelta = TimeDelta::zero();
        let mut break_time: TimeDelta = TimeDelta::zero();
        let mut off_shift_time: TimeDelta = TimeDelta::zero();
        let mut toolbox_time: TimeDelta = TimeDelta::zero();
        let mut non_productive_time: TimeDelta = TimeDelta::zero();

        dbg!(self.operational_solutions.0.len());
        dbg!(self.operational_non_productive.0.len());
        dbg!(self.operational_parameters.len());
        let all_events = operational_events
            .iter()
            .chain(&self.operational_non_productive.0);

        for (index, assignment) in all_events.clone().enumerate() {
            match &assignment.event_type {
                OperationalEvents::WrenchTime(time_interval) => {
                    wrench_time += time_interval.duration();
                    current_time += time_interval.duration();
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
                }
            }
        }

        // assert_eq!(current_time, self.availability.end_date);

        equality_between_time_interval_and_assignments(all_events.clone().collect::<Vec<_>>());

        assert!(is_assignments_in_bounds(
            all_events.clone().collect::<Vec<_>>(),
            &self.availability
        ));

        assert!(no_overlap(all_events.collect::<Vec<_>>()));

        let total_time =
            (wrench_time + break_time + off_shift_time + toolbox_time + non_productive_time);
        assert_eq!(total_time, self.availability.duration());
        self.objective_value = (wrench_time).num_seconds() as f64
            / (wrench_time + break_time + toolbox_time + non_productive_time).num_seconds() as f64;
    }

    fn schedule(&mut self) {
        for (operation_id, operational_parameter) in &self.operational_parameters {
            let start_time = determine_first_available_start_time(
                operational_parameter,
                &self.operational_solutions,
            );

            let assignments =
                self.determine_wrench_time_assignment(start_time, operational_parameter);
            self.operational_solutions
                .try_insert(*operation_id, assignments);
        }

        // fill the schedule
        self.operational_non_productive.0.clear();
        let mut current_time = self.availability.start_date;

        loop {
            match self
                .operational_solutions
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
                    assert_ne!(
                        &operational_event,
                        &OperationalEvents::WrenchTime(TimeInterval::default())
                    );
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
                    assert_ne!(
                        &operational_event,
                        &OperationalEvents::WrenchTime(TimeInterval::default())
                    );
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
    }

    fn unschedule(&mut self, work_order_and_activity_number: Self::SchedulingUnit) {
        let unscheduled_operational_solution = self
            .operational_solutions
            .0
            .iter()
            .find(|operational_solution| {
                operational_solution.0 == work_order_and_activity_number.0
                    && operational_solution.1 == work_order_and_activity_number.1
            })
            .take();
        unscheduled_operational_solution.expect("There was nothing in the operational solution");
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse, Self::Error> {
        todo!()
    }

    fn update_time_state(
        &mut self,
        _message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse, Self::Error> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        _message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse, Self::Error> {
        todo!();
    }
}

impl OperationalAlgorithm {
    fn determine_wrench_time_assignment(
        &self,
        start_time: DateTime<Utc>,
        operational_parameter: &OperationalParameter,
    ) -> Vec<Assignment> {
        let mut assigned_work: Vec<Assignment> = vec![];
        let mut remaining_combined_work = operational_parameter.operation_time_delta.clone();
        let mut current_time = start_time;
        while !remaining_combined_work.is_zero() {
            if self.break_interval.contains(&current_time) {
                current_time += self.break_interval.end - current_time.time();
                assert!(self.break_interval.end - current_time.time() >= TimeDelta::zero());
            } else if self.off_shift_interval.contains(&current_time) {
                current_time += self.off_shift_interval.end - current_time.time();
                assert!(self.toolbox_interval.end - current_time.time() >= TimeDelta::zero());
            } else if self.toolbox_interval.contains(&current_time) {
                current_time += self.toolbox_interval.end - current_time.time();
                assert!(self.break_interval.end - current_time.time() >= TimeDelta::zero());
            };

            let next_event = self.determine_next_event(&current_time);

            if next_event.0.is_zero() {
                let finish_time = current_time + next_event.1.time_delta();
                assigned_work.push(Assignment::new(next_event.1, current_time, finish_time));
                current_time = finish_time;
            } else if next_event.0 < remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime(TimeInterval::new(
                        current_time.time(),
                        (current_time + next_event.0).time(),
                    )),
                    current_time,
                    current_time + next_event.0,
                ));
                current_time += next_event.0 + next_event.1.time_delta();
                remaining_combined_work -= next_event.0;
            } else if next_event.0 >= remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime(TimeInterval::new(
                        current_time.time(),
                        (current_time + remaining_combined_work).time(),
                    )),
                    current_time,
                    current_time + remaining_combined_work,
                ));
                current_time += next_event.0;
                remaining_combined_work = TimeDelta::zero();
            }
        }
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

        vec![break_diff, toolbox_diff, off_shift_diff]
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
        let operational_solutions: Vec<(WorkOrderNumber, ActivityNumber)> = self
            .operational_solutions
            .0
            .choose_multiple(rng, number_of_activities)
            .map(|operational_solution| (operational_solution.0, operational_solution.1))
            .collect();

        for operational_solution in operational_solutions {
            self.unschedule((operational_solution.0, operational_solution.1));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalEvents {
    WrenchTime(TimeInterval),
    Break(TimeInterval),
    Toolbox(TimeInterval),
    OffShift(TimeInterval),
    NonProductiveTime(TimeInterval),
}

impl OperationalEvents {
    fn time_delta(&self) -> TimeDelta {
        match self {
            Self::WrenchTime(time_interval) => time_interval.duration(),
            Self::Break(time_interval) => time_interval.duration(),
            Self::Toolbox(time_interval) => time_interval.duration(),
            Self::OffShift(time_interval) => time_interval.duration(),
            Self::NonProductiveTime(time_interval) => time_interval.duration(),
        }
    }
    fn start_time(&self) -> NaiveTime {
        match self {
            Self::WrenchTime(time_interval) => time_interval.start,
            Self::Break(time_interval) => time_interval.start,
            Self::Toolbox(time_interval) => time_interval.start,
            Self::OffShift(time_interval) => time_interval.start,
            Self::NonProductiveTime(time_interval) => time_interval.start,
        }
    }
    fn finish_time(&self) -> NaiveTime {
        match self {
            Self::WrenchTime(time_interval) => time_interval.end,
            Self::Break(time_interval) => time_interval.end,
            Self::Toolbox(time_interval) => time_interval.end,
            Self::OffShift(time_interval) => time_interval.end,
            Self::NonProductiveTime(time_interval) => time_interval.end,
        }
    }
}

fn determine_first_available_start_time(
    operational_parameter: &OperationalParameter,
    operational_solutions: &OperationalSolutions,
) -> DateTime<Utc> {
    for operational_solution in operational_solutions.0.windows(2) {
        let start_of_interval = match &operational_solution[0].2 {
            Some(operational_solution) => operational_solution.assignments.last().unwrap().finish,
            None => break,
        };

        let end_of_interval = match &operational_solution[1].2 {
            Some(operational_solution) => operational_solution.assignments.first().unwrap().start,
            None => continue,
        };

        if operational_parameter.end_window.min(end_of_interval)
            - operational_parameter.start_window.max(start_of_interval)
            > operational_parameter.operation_time_delta
        {
            return operational_parameter.start_window.max(start_of_interval);
        }
    }
    operational_parameter.start_window
}

fn no_overlap(events: Vec<&Assignment>) -> bool {
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

fn is_assignments_in_bounds(events: Vec<&Assignment>, availability: &Availability) -> bool {
    for event in events {
        if event.start < availability.start_date {
            dbg!(event, availability);
            return false;
        }
        if availability.end_date < event.finish {
            dbg!(event, availability);
            return false;
        }
    }
    true
}

fn equality_between_time_interval_and_assignments(all_events: Vec<&Assignment>) {
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
    use proptest::prelude::*;

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
