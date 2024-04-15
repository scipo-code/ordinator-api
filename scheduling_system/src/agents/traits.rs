use std::fmt;

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

impl<T> AlgorithmState<T> {
    pub fn infeasible_cases_mut(&mut self) -> Option<&mut T> {
        match self {
            AlgorithmState::Feasible => None,
            AlgorithmState::Infeasible(infeasible_cases) => Some(infeasible_cases),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ConstraintState<Reason> {
    Feasible,
    Infeasible(Reason),
}

impl<Reason> fmt::Display for ConstraintState<Reason>
where
    Reason: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintState::Feasible => write!(f, "FEASIBLE"),
            ConstraintState::Infeasible(reason) => write!(f, "{}", reason),
        }
    }
}
