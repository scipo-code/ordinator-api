pub trait LargeNeighborHoodSearch {
    type SchedulingMessage;
    type ResourceMessage;
    type TimeMessage;

    type Error;

    fn calculate_objective_value(&mut self);

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
    type InfeasibleCases;

    fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases>;
}

pub enum AlgorithmState<T> {
    Feasible,
    Infeasible(T),
}
