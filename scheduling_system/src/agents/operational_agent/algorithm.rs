use std::collections::HashMap;

use chrono::{DateTime, Utc};
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
        operational_response_time::OperationalTimeResponse,
    },
};

use crate::agents::traits::LargeNeighborHoodSearch;

use super::Assigned;

pub struct OperationalAlgorithm {
    pub objective_value: f64,
    pub operational_solution: HashMap<(WorkOrderNumber, ActivityNumber), OperationalSolution>,
    pub operational_parameters: HashMap<(WorkOrderNumber, ActivityNumber), OperationalParameters>,
    pub availability: Option<Availability>,
}

impl OperationalAlgorithm {
    pub fn new() -> Self {
        Self {
            objective_value: f64::INFINITY,
            operational_solution: HashMap::new(),
            operational_parameters: HashMap::new(),
            availability: None,
        }
    }

    pub fn insert_optimized_operation(
        &mut self,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        operational_parameters: OperationalParameters,
        operational_solution: OperationalSolution,
    ) {
        self.operational_parameters
            .insert((work_order_number, activity_number), operational_parameters);
        self.operational_solution
            .insert((work_order_number, activity_number), operational_solution);
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

pub struct OperationalParameters {
    work: f64,
    preparation: f64,
    start_window: DateTime<Utc>,
    end_window: DateTime<Utc>,
}

impl OperationalParameters {
    pub fn new(
        work: f64,
        preparation: f64,
        start_window: DateTime<Utc>,
        end_window: DateTime<Utc>,
    ) -> Self {
        Self {
            work,
            preparation,
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
        todo!()
    }

    fn schedule(&mut self) {
        todo!()
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
