use std::fmt;

use shared_messages::{models::work_order::WorkOrderNumber, AlgorithmState};

pub trait LargeNeighborHoodSearch {
    type SchedulingMessage;
    type SchedulingResponse;
    type ResourceMessage;
    type ResourceResponse;
    type TimeMessage;
    type TimeResponse;

    type Error;

    fn calculate_objective_value(&mut self);

    fn schedule(&mut self);

    fn unschedule(&mut self, message: WorkOrderNumber);

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingMessage,
    ) -> Result<Self::SchedulingResponse, Self::Error>;

    fn update_time_state(
        &mut self,
        message: Self::TimeMessage,
    ) -> Result<Self::TimeResponse, Self::Error>;

    fn update_resources_state(
        &mut self,
        message: Self::ResourceMessage,
    ) -> Result<Self::ResourceResponse, Self::Error>;
}

/// TestAlgorithm is a trait that all algorithms should implement. Running `feasible()` tests if the solution
/// violates any constraints of the problem, or if the objective is not correctly calcalated.
pub trait TestAlgorithm {
    type InfeasibleCases;

    fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases>;
}
