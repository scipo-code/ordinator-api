use anyhow::Context;
use anyhow::Result;

use std::fmt::Debug;
use std::sync::MutexGuard;

use shared_types::scheduling_environment::work_order::WorkOrderActivity;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::operational_agent::algorithm::operational_solution::MarginalFitness;
use super::AlgorithmBuilder;
use super::StateLink;

/// This trait will be crucial for making this whole thing work correctly.
/// I think that the best approach will be to make only a single message
/// and then have that as an enum. Then we should have the 'update_shared_solution'
/// as a function to make sure that if the state of the other agents have
/// changed that we update that correctly in the solution.
///
/// QUESTION:
/// Should you make this function on the correct kind of
pub trait ActorBasedLargeNeighborhoodSearch
where
    Self: AlgorithmUtils,
{
    type Options;

    fn run_lns_iteration(&mut self) -> Result<()> {
        self.update_based_on_shared_solution()?;

        let current_solution = self.clone_algorithm_solution();

        self.unschedule()
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

    fn make_atomic_pointer_swap(&self);

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType<Self::ObjectiveValue>>;

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self) -> Result<()>;

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

pub trait AlgorithmUtils {
    type Parameters: Parameters;
    type ObjectiveValue;
    type Sol: Solution<ObjectiveValue = Self::ObjectiveValue> + Debug + Clone;
    type I: Default;

    fn builder() -> AlgorithmBuilder<Self::Sol, Self::Parameters, Self::I>;

    fn load_shared_solution(&mut self);

    fn clone_algorithm_solution(&self) -> Self::Sol;

    fn swap_solution(&mut self, solution: Self::Sol);

    // WARN
    // You may have to reintroduce this.
    // fn update_objective_value(&mut self, objective_value: Self::ObjectiveValue);
}

#[allow(dead_code)]
pub enum ObjectiveValueType<O> {
    Better(O),
    Worse,
    Force,
}

trait ObjectiveValue {}

// WARN
// More complex logic will be needed here for later. Start with this kind
// of implementation and then continue to make the most of it. I think
// that it is a better choice to quickly make this interface and then
// change afterwards.
//
// This means that this should not have a `new` function, but instead
//
pub trait Parameters
where
    Self: Sized,
{
    type Key;
    type Options;

    fn new(
        id: &Id,
        options: Self::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>;

    /// WARNING
    /// This method can become extremely complex in a practical setting.
    /// You should do.
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    );

    // TODO [ ]
    // Add methods for updating configurations.
}

pub trait Solution {
    type ObjectiveValue;
    type Parameters;

    // QUESTION
    // Is this a good idea to create the Solution? I actually believe that it
    // is!
    fn new(parameters: &Self::Parameters) -> Self;

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue);
}

pub trait MessageHandler {
    type Req;
    type Res;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()>;

    fn handle_request_message(&mut self, request_message: Self::Req) -> Result<Self::Res>;
}

/// You should most likely remove this and insert something else instead. I think
#[allow(dead_code)]
pub trait GetMarginalFitness {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness>;
}
