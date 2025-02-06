use anyhow::Result;

/// This trait will be crucial for making this whole thing work correctly.
/// I think that the best approach will be to make only a single message
/// and then have that as an enum. Then we should have the 'update_shared_solution'
/// as a function to make sure that if the state of the other agents have
/// changed that we update that correctly in the solution.
#[allow(dead_code)]
pub trait ActorBasedLargeNeighborhoodSearch {

    type MessageRequest;

    type MessageResponse;

    type SchedulingUnit;

    type Options;

    fn run_lns_iteration(&mut self, options: &Self::Options) -> Result<()> {
        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        self.check_messages();

        self.load_shared_solution();

        // self.updated_shared_solution(&mut self)?;

        let current_solution = self.clone_algorithm_solution().clone();

        self.unschedule(options)?;

        self.schedule()?;

        let objective_value_type = self.calculate_objective_value()?;

        match objective_value_type {
            ObjectiveValueType::Better => todo!(),
            ObjectiveValueType::Worse => todo!(),
            ObjectiveValueType::Force => todo!(),
        }

        Ok(())
    }

    // fn load_shared_solution(&mut self) {
    //     self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    // }

    fn clone_algorithm_solution(&self) -> impl Solution;

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType>;

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self, unschedule_options: Self::Options) -> Result<()>;

    fn update_based_on_message(&mut self, Self::MessageRequest) -> Result<Self::MessageResponse>;

    // fn update_shared_solution(&mut self) -> Result<()>;
}

pub trait Solution: Clone {}

pub enum ObjectiveValueType {
    Better,
    Worse,
    Force,
}
