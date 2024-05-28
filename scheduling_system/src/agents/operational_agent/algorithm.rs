use std::collections::HashMap;

use chrono::{DateTime, NaiveTime, TimeDelta, Utc};
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

use super::Assigned;

pub struct OperationalAlgorithm {
    pub objective_value: f64,
    pub operational_solution: Vec<(WorkOrderNumber, ActivityNumber, OperationalSolution)>,
    pub operational_parameters: HashMap<(WorkOrderNumber, ActivityNumber), OperationalParameter>,
    pub availability: Availability,
    pub shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval, 
}


impl OperationalAlgorithm {
    pub fn new(availability: Availability, shift_interval: TimeInterval, break_interval: TimeInterval, toolbox_interval: TimeInterval) -> Self {
        Self {
            objective_value: f64::INFINITY,
            operational_solution: vec![],
            operational_parameters: HashMap::new(),
            availability,
            shift_interval,
            break_interval,
            toolbox_interval,
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

pub struct OperationalSolution {
    assigned: Assigned,
    assignments: Vec<Assignment>,
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
            let combined_time =
                3600.0 * (work + preparation);
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

type OperationalSolutions = Vec<(WorkOrderNumber, ActivityNumber, OperationalSolution)>;

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
        let old_objective = self.objective_value.clone();

        let operational_solutions = self.operational_solution;
        for (operation_id, operational_parameter) in &self.operational_parameters {
            let mut time_start = operational_parameter.start_window;


            match determine_first_available_time_slot(
                operatonal_parameter,
                operational_solutions,
            ) {
                Some(start_time) => self.schedule_operation(
                    start_time,
                    operational_parameter,
                ),
                None => return operation_id,
            }
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
    fn schedule_operation(
        &mut self,
        start_time: DateTime<Utc>,
        operational_parameter: &OperationalParameter,
    ) {

        let mut remaining_combined_work = operational_parameter.operation_time_delta.clone();
        while remaining_combined_work != 0 {
            

            let overlap: Option<PossibleOverlaps> = if self.break_interval.contains(start_time) {
                Some(PossibleOverlaps::Break)
            } else if self.shift_interval.contains(start_time) {
                Some(PossibleOverlaps::OffShift)
            } else if self.toolbox_interval.contains(start_time) {
                Some(PossibleOverlaps::Toolbox)
            } else {
                None
            };
            
            
            
            let end_time = start_time
            Assignment { start_time, 

        }


        
    }
}

enum PossibleOverlaps {
    Break,
    Toolbox,
    OffShift,
}

fn determine_first_available_time_slot(
    operational_parameter: OperationalParameter,
    operational_solutions: OperationalSolutions,
) -> Option<DateTime<Utc>> {
    for operational_solution in operational_solutions.windows(2) {
        let start_of_interval = operational_solution[0].2.assignments.last().unwrap().finish;
        let end_of_interval = operational_solution[1].2.assignments.first().unwrap().start;

        if operational_parameter.end_window.min(end_of_interval) - operational_parameter.start_window.max(start_of_interval)
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
