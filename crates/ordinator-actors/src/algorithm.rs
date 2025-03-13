use anyhow::Result;
use arc_swap::Guard;

use std::sync::Arc;
use std::{fmt::Debug, sync::MutexGuard};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::traits::AbLNSUtils;
use super::traits::Parameters;
use super::traits::Solution;
use super::ArcSwapSharedSolution;
use super::SharedSolution;

// QUESTION
// You are making a lot of fields public here. I do not think that
// is a good idea. Why you should use a method to retain and remove
// solutions. And this is the only way of doing it.
pub struct Algorithm<S, P, I>
where
    S: Solution,
    P: Parameters,
{
    pub(super) id: Id,
    pub(super) solution_intermediate: I,
    pub(super) solution: S,
    pub(super) parameters: P,
    pub(super) arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub(super) loaded_shared_solution: Guard<Arc<SharedSolution>>,
}

pub struct AlgorithmBuilder<S, P, I>
where
    S: Solution,
    P: Parameters,
{
    id: Option<Id>,
    solution_intermediate: I,
    solution: Option<S>,
    parameters: Option<P>,
    arc_swap_shared_solution: Option<Arc<ArcSwapSharedSolution>>,
    loaded_shared_solution: Option<Guard<Arc<SharedSolution>>>,
}

impl<S, P, I> Algorithm<S, P, I>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
{
    pub fn builder() -> AlgorithmBuilder<S, P, I> {
        AlgorithmBuilder {
            id: None,
            solution_intermediate: I::default(),
            solution: None,
            parameters: None,
            arc_swap_shared_solution: None,
            loaded_shared_solution: None,
        }
    }
}
impl<S, P, I> AbLNSUtils for Algorithm<S, P, I>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
{
    type SolutionType = S;
    fn clone_algorithm_solution(&self) -> S {
        self.solution.clone()
    }

    fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    fn swap_solution(&mut self, solution: S) {
        self.solution = solution;
    }

    fn update_objective_value(
        &mut self,
        objective_value: <Self::SolutionType as Solution>::ObjectiveValue,
    ) {
        self.solution.update_objective_value(objective_value);
    }
}

impl<S, P, I> AlgorithmBuilder<S, P, I>
where
    S: Solution<Parameters = P>,
    P: Parameters,
    I: Default,
{
    pub fn build(self) -> Algorithm<S, P, I> {
        Algorithm {
            id: self.id.unwrap(),
            solution_intermediate: self.solution_intermediate,
            solution: self.solution.unwrap(),
            parameters: self.parameters.unwrap(),
            arc_swap_shared_solution: self.arc_swap_shared_solution.unwrap(),
            loaded_shared_solution: self.loaded_shared_solution.unwrap(),
        }
    }
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
    // This should call the relevant method instead of the
    pub fn solution(mut self) -> Self {
        let parameters = self
            .parameters
            .as_ref()
            .expect("You must call `parameters` before `solution`");
        let solution = S::new(parameters);
        self.solution = Some(solution);
        self
    }

    pub fn parameters(
        mut self,
        options: &P::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let parameters = Parameters::new(
            &self.id.as_ref().expect("Call `id()` build method first"),
            options,
            scheduling_environment,
        )?;
        self.parameters = Some(parameters);
        Ok(self)
    }
    pub fn arc_swap_shared_solution(
        mut self,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    ) -> Self {
        self.arc_swap_shared_solution = Some(arc_swap_shared_solution);
        self.loaded_shared_solution = Some(
            self.arc_swap_shared_solution
                .as_ref()
                .expect("Set the `arc_swap` field first")
                .0
                .load(),
        );
        self
    }
}

// TODO [x]
// Where should this be moved to? I am not really sure! I think that the best place is the `Algorithm`
// no I think it is the `ordinator-actors` crate
pub enum LoadOperation {
    Add,
    Sub,
}
