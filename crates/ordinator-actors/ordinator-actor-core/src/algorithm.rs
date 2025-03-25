use std::fmt::Debug;
use std::sync::Arc;
use std::sync::MutexGuard;

use anyhow::Result;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use crate::traits::AbLNSUtils;
use crate::traits::ActorBasedLargeNeighborhoodSearch;

// pub type SharedSolution = SharedSolution<

// QUESTION
// You are making a lot of fields public here. I do not think that
// is a good idea. Why you should use a method to retain and remove
// solutions. And this is the only way of doing it.
pub struct Algorithm<S, P, I, Ss>
where
    S: Solution,
    P: Parameters,
    Ss: SharedSolutionTrait,
{
    pub id: Id,
    pub solution_intermediate: I,
    pub solution: S,
    pub parameters: P,
    pub arc_swap_shared_solution: Arc<ArcSwap<Ss>>,
    pub loaded_shared_solution: Guard<Arc<Ss>>,
}

pub struct AlgorithmBuilder<S, P, I, Ss>
where
    S: Solution,
    P: Parameters,
    Ss: SharedSolutionTrait,
{
    id: Option<Id>,
    solution_intermediate: I,
    solution: Option<S>,
    parameters: Option<P>,
    arc_swap_shared_solution: Option<Arc<ArcSwap<Ss>>>,
    loaded_shared_solution: Option<Guard<Arc<Ss>>>,
}

impl<S, P, I, Ss> Algorithm<S, P, I, Ss>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
    Ss: SharedSolutionTrait,
{
    pub fn builder() -> AlgorithmBuilder<S, P, I, Ss>
    {
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
impl<S, P, I, Ss> AbLNSUtils for Algorithm<S, P, I, Ss>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
    Ss: SharedSolutionTrait,
{
    type SolutionType = S;

    fn clone_algorithm_solution(&self) -> S
    {
        self.solution.clone()
    }

    fn load_shared_solution(&mut self)
    {
        self.loaded_shared_solution = self.arc_swap_shared_solution.load();
    }

    fn swap_solution(&mut self, solution: S)
    {
        self.solution = solution;
    }

    fn update_objective_value(
        &mut self,
        objective_value: <Self::SolutionType as Solution>::ObjectiveValue,
    )
    {
        self.solution.update_objective_value(objective_value);
    }
}

impl<S, P, I, Ss> AlgorithmBuilder<S, P, I, Ss>
where
    S: Solution<Parameters = P>,
    P: Parameters,
    I: Default,
    Ss: SharedSolutionTrait,
{
    pub fn build<Alg>(self) -> Alg
    where
        Algorithm<S, P, I, Ss>: Into<Alg>,
    {
        let algorithm_inner = Algorithm {
            id: self.id.unwrap(),
            solution_intermediate: self.solution_intermediate,
            solution: self.solution.unwrap(),
            parameters: self.parameters.unwrap(),
            arc_swap_shared_solution: self.arc_swap_shared_solution.unwrap(),
            loaded_shared_solution: self.loaded_shared_solution.unwrap(),
        };

        algorithm_inner.into()
    }

    pub fn id(mut self, id: Id) -> Self
    {
        self.id = Some(id);
        self
    }

    // This should call the relevant method instead of the
    pub fn solution(mut self) -> Self
    {
        let parameters = self
            .parameters
            .as_ref()
            .expect("You must call `parameters` before `solution`");
        let solution = S::new(parameters);
        self.solution = Some(solution);
        self
    }

    // This is a needless level of indirection. You should be careful of this type
    // of thing. The issue here is what we should do about this.
    // What should happen to this function? I think that the best place to have
    // there kind of things
    pub fn parameters(
        mut self,
        options: P::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let parameters = P::from_source(
            self.id.as_ref().expect("Call `id()` build method first"),
            options,
            scheduling_environment,
        )?;
        self.parameters = Some(parameters);
        Ok(self)
    }

    pub fn arc_swap_shared_solution(mut self, arc_swap_shared_solution: Arc<ArcSwap<Ss>>) -> Self
    where
        Ss: SharedSolutionTrait,
    {
        self.arc_swap_shared_solution = Some(arc_swap_shared_solution);
        self.loaded_shared_solution = Some(
            self.arc_swap_shared_solution
                .as_ref()
                .expect("Set the `arc_swap` field first")
                .load(),
        );
        self
    }
}

// TODO [x]
// Where should this be moved to? I am not really sure! I think that the best
// place is the `Algorithm` no I think it is the `ordinator-actors` crate
pub enum LoadOperation
{
    Add,
    Sub,
}
