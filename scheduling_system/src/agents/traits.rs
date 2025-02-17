use anyhow::{Context, Result};
use std::fmt::Debug;

/// This trait will be crucial for making this whole thing work correctly.
/// I think that the best approach will be to make only a single message
/// and then have that as an enum. Then we should have the 'update_shared_solution'
/// as a function to make sure that if the state of the other agents have
/// changed that we update that correctly in the solution.
pub trait ActorBasedLargeNeighborhoodSearch {
    type MessageRequest;
    type MessageResponse;
    type Solution: Debug;
    type ObjectiveValue;
    type Options;

    fn run_lns_iteration(&mut self, options: &mut Self::Options) -> Result<()> {
        self.update_based_on_shared_solution()?;

        let current_solution = self.clone_algorithm_solution();

        self.unschedule(options)
            .with_context(|| format!("{:#?}", current_solution))?;

        self.schedule()
            .with_context(|| format!("Could not schedule\n{:#?}", current_solution))?;

        let objective_value_type = self.calculate_objective_value()?;

        match objective_value_type {
            ObjectiveValueType::Better(objective_value) => {
                self.update_objective_value(objective_value);
                self.make_atomic_pointer_swap();
            }
            ObjectiveValueType::Worse => self.swap_solution(current_solution),
            ObjectiveValueType::Force => todo!(),
        }

        Ok(())
    }

    fn clone_algorithm_solution(&self) -> Self::Solution;

    fn swap_solution(&mut self, solution: Self::Solution);

    fn make_atomic_pointer_swap(&self);

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType<Self::ObjectiveValue>>;

    fn update_objective_value(&mut self, objective_value: Self::ObjectiveValue);

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self, unschedule_options: &mut Self::Options) -> Result<()>;

    /// This method is for updating the algorithm based on external inputs and
    /// the shared solution. That means that this method has to look at relevant
    /// state in the others `Agent`s and incorporate that and handled changes in
    /// parameters coming from external inputs.
    fn update_based_on_shared_solution(&mut self) -> Result<()> {
        self.load_shared_solution();

        let state_change = self.incorporate_shared_state()?;

        if state_change {
            self.calculate_objective_value()?;
            self.make_atomic_pointer_swap();
        }

        Ok(())
    }

    fn incorporate_shared_state(&mut self) -> Result<bool>;
}

#[allow(dead_code)]
pub enum ObjectiveValueType<O> {
    Better(O),
    Worse,
    Force,
}

trait ObjectiveValue {}
