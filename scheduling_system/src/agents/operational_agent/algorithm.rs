use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};
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

pub struct OperationalAlgorithm {
    pub objective_value: f64,
    pub operational_solutions: OperationalSolutions,
    pub operational_parameters: HashMap<(WorkOrderNumber, ActivityNumber), OperationalParameter>,
    pub availability: Availability,
    pub shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

impl OperationalAlgorithm {
    pub fn new(operational_configuration: OperationalConfiguration) -> Self {
        Self {
            objective_value: f64::INFINITY,
            operational_solutions: OperationalSolutions(Vec::new()),
            operational_parameters: HashMap::new(),
            availability: operational_configuration.availability,
            shift_interval: operational_configuration.shift_interval,
            break_interval: operational_configuration.break_interval,
            toolbox_interval: operational_configuration.toolbox_interval,
        }
    }

    pub fn insert_optimized_operation(
        &mut self,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        operational_parameters: OperationalParameter,
        operational_solution: OperationalSolution,
    ) {
        self.operational_parameters
            .insert((work_order_number, activity_number), operational_parameters);
    }
}

pub struct OperationalSolutions(
    pub Vec<(WorkOrderNumber, ActivityNumber, Option<OperationalSolution>)>,
);

impl OperationalSolutions {
    fn try_insert(
        &mut self,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        assignments: Vec<Assignment>,
    ) {
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

                self.0.insert(
                    index + 1,
                    (
                        work_order_number,
                        activity_number,
                        Some(operational_solution),
                    ),
                );
                break;
            }
        }
        self.0.push((work_order_number, activity_number, None));
    }
}

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
}

pub struct Assignment {
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

impl OperationalSolution {
    pub fn new(assigned: Assigned, assignments: Vec<Assignment>) -> Self {
        Self {
            assigned,
            assignments,
        }
    }
}

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

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        // Here we should determine the objective based on the highest needed skill. Meaning that a MTN-TURB should not bid highly
        // on a MTN-MECH job. I think that this will be very interesting to solve.
        todo!()
    }

    fn schedule(&mut self) {
        // let old_objective = self.objective_value.clone();

        for (operation_id, operational_parameter) in &self.operational_parameters {
            let start_time_option = determine_first_available_start_time(
                operational_parameter,
                &self.operational_solutions,
            );

            match start_time_option {
                Some(start_time) => {
                    let assignments = self.determine_assignment(start_time, operational_parameter);
                    self.operational_solutions.try_insert(
                        operation_id.0,
                        operation_id.1,
                        assignments,
                    );
                }
                None => continue,
            };

            // If the operation does not fit in the schedule it should be scheduled in the next round of the LNS optimization.
            // The operational agent is different in that there can be no penalty.
        }
        // self.operational_solution
    }

    fn unschedule(&mut self, _message: WorkOrderNumber) {
        todo!()
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
        todo!()
    }
}

impl OperationalAlgorithm {
    fn determine_assignment(
        &self,
        start_time: DateTime<Utc>,
        operational_parameter: &OperationalParameter,
    ) -> Vec<Assignment> {
        let mut assigned_work: Vec<Assignment> = vec![];
        let mut remaining_combined_work = operational_parameter.operation_time_delta.clone();
        while !remaining_combined_work.is_zero() {
            let mut current_time = start_time;

            if self.break_interval.contains(current_time) {
                current_time += self.break_interval.end - current_time.time();
            } else if self.shift_interval.contains(current_time) {
                current_time += self.toolbox_interval.end - current_time.time();
            } else if self.toolbox_interval.contains(current_time) {
                current_time += self.shift_interval.end - current_time.time();
            };

            let next_event = self.next_event(current_time);

            if next_event.0 < remaining_combined_work {
                assigned_work.push(Assignment {
                    start: current_time,
                    finish: current_time + next_event.0,
                });
                current_time += next_event.0 + next_event.1.time_delta();
                remaining_combined_work -= next_event.0;
            } else if next_event.0 >= remaining_combined_work {
                assigned_work.push(Assignment {
                    start: current_time,
                    finish: current_time + remaining_combined_work,
                });
                current_time += next_event.0;
                remaining_combined_work = TimeDelta::zero();
            }
        }
        assigned_work
    }

    fn next_event(&self, current_time: DateTime<Utc>) -> (TimeDelta, OperationalEvents) {
        let break_diff = (
            self.break_interval.start - current_time.time(),
            OperationalEvents::Break(self.break_interval.clone()),
        );
        let toolbox_diff = (
            self.toolbox_interval.start - current_time.time(),
            OperationalEvents::Toolbox(self.toolbox_interval.clone()),
        );
        let off_shift_diff = (
            self.shift_interval.start - current_time.time(),
            OperationalEvents::OffShift(self.shift_interval.clone()),
        );

        vec![break_diff, toolbox_diff, off_shift_diff]
            .iter()
            .filter(|&diff_event| diff_event.0.num_seconds() >= 0)
            .min_by_key(|&diff_event| diff_event.0.num_seconds())
            .cloned()
            .unwrap()
    }
}

#[derive(Clone)]
enum OperationalEvents {
    Break(TimeInterval),
    Toolbox(TimeInterval),
    OffShift(TimeInterval),
}

enum OperationalOperationState {
    Feasible,
    Infeasible((WorkOrderNumber, ActivityNumber)),
}

impl OperationalEvents {
    fn time_delta(&self) -> TimeDelta {
        match self {
            Self::Break(time_interval) => time_interval.end - time_interval.start,
            Self::Toolbox(time_interval) => time_interval.end - time_interval.start,
            Self::OffShift(time_interval) => time_interval.end - time_interval.start,
        }
    }
}

fn determine_first_available_start_time(
    operational_parameter: &OperationalParameter,
    operational_solutions: &OperationalSolutions,
) -> Option<DateTime<Utc>> {
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
            return Some(operational_parameter.start_window.max(start_of_interval));
        }
    }
    None
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
