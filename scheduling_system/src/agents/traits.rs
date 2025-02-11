use anyhow::Result;

/// This trait will be crucial for making this whole thing work correctly.
/// I think that the best approach will be to make only a single message
/// and then have that as an enum. Then we should have the 'update_shared_solution'
/// as a function to make sure that if the state of the other agents have
/// changed that we update that correctly in the solution.
pub trait ActorBasedLargeNeighborhoodSearch {
    type MessageRequest;
    type MessageResponse;
    type Solution;
    type Options;

    fn run_lns_iteration(&mut self, options: &mut Self::Options) -> Result<()> {
        self.load_shared_solution();

        self.update_based_on_shared_solution()?;

        let current_solution = self.clone_algorithm_solution();

        self.unschedule(options)?;

        self.schedule()?;

        let objective_value_type = self.calculate_objective_value()?;

        match objective_value_type {
            ObjectiveValueType::Better => {
                // FIX
                // This can be Solved be making the Algorithm generic. I think that is a really good idea.
                // You have to prepare the project plan now though. And then later go running. Do you need
                // to go to work today?
                self.make_atomic_pointer_swap();
            }
            ObjectiveValueType::Worse => self.swap_solution(current_solution),
            ObjectiveValueType::Force => todo!(),
        }

        Ok(())
    }

    fn load_shared_solution(&mut self);

    fn clone_algorithm_solution(&self) -> Self::Solution;

    fn swap_solution(&mut self, solution: Self::Solution);

    fn make_atomic_pointer_swap(&self);

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType>;

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self, unschedule_options: &mut Self::Options) -> Result<()>;

    fn update_based_on_shared_solution(&mut self) -> Result<()>;
}

#[allow(dead_code)]
pub enum ObjectiveValueType {
    Better,
    Worse,
    Force,
}
