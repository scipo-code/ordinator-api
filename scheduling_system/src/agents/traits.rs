pub trait LargeNeighborHoodSearch {
    type SchedulingMessage;
    type ResourceMessage;
    type TimeMessage;

    type Error;

    fn objective_value(&self) -> f64;

    fn schedule(&mut self);

    fn unschedule(&mut self, message: u32);

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingMessage,
    ) -> Result<String, Self::Error>;

    fn update_time_state(&mut self, message: Self::TimeMessage) -> Result<String, Self::Error>;

    fn update_resources_state(
        &mut self,
        message: Self::ResourceMessage,
    ) -> Result<String, Self::Error>;
}

/// TestAlgorithm is a trait that all algorithms should implement. Running `feasible()` tests if the solution
/// violates any constraints of the problem, or if the objective is not correctly calcalated.
pub trait TestAlgorithm {
    fn determine_algorithm_state(&self) -> AlgorithmState;
}

pub enum AlgorithmState {
    Feasible,
    Infeasible,
}
