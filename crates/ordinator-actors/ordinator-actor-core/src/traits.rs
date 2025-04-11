use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use arc_swap::Guard;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

/// This trait will be crucial for making this whole thing work correctly.
/// I think that the best approach will be to make only a single message
/// and then have that as an enum. Then we should have the
/// 'update_shared_solution' as a function to make sure that if the state of the
/// other agents have changed that we update that correctly in the solution.
///
/// QUESTION:
/// Should you make this function on the correct kind of
// QUESTION
// Do you even need this trait? No you do not... This can simply be
// created using the generic structure? No you need the generic
// structure. There is no way around it. I think that the best
// thing to do here is making the.
// The high level thing only requires the actor specification, and only the
// actor should know about that. I really do not think that I need an additional
// break here. I do not see what other approach.
pub trait ActorBasedLargeNeighborhoodSearch
{
    type Algorithm: AbLNSUtils;
    type Options;

    fn run_lns_iteration(
        &mut self,
        configurations: Guard<Arc<SystemConfigurations>>,
        id: &Id,
    ) -> Result<()>
    {
        let options = &Self::derive_options(&configurations, id);

        // You still have the same problem. Why do you keep running in circles? I do not
        // understand it. You have to fix this. You will work longer hours.
        self.update_based_on_shared_solution(options)?;

        // But that means that we cannot code this
        let current_solution = self.algorithm_util_methods().clone_algorithm_solution();

        self.unschedule()
            .with_context(|| format!("{:#?}", current_solution))?;

        self.schedule()
            .with_context(|| format!("Could not schedule\n{:#?}", current_solution))?;

        let objective_value_type = self.calculate_objective_value(options)?;

        match objective_value_type {
            ObjectiveValueType::Better(objective_value) => {
                self.algorithm_util_methods()
                    .update_objective_value(objective_value);
                self.make_atomic_pointer_swap();
            }
            ObjectiveValueType::Worse => self
                .algorithm_util_methods()
                .swap_solution(current_solution),
            ObjectiveValueType::Force => todo!(),
        }

        Ok(())
    }

    fn derive_options(configurations: &Guard<Arc<SystemConfigurations>>, id: &Id) -> Self::Options;

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm;

    fn make_atomic_pointer_swap(&mut self);

    fn calculate_objective_value(
        &mut self,
        options: &Self::Options,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    >;

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self) -> Result<()>;

    /// This method is for updating the algorithm based on external inputs and
    /// the shared solution. That means that this method has to look at relevant
    /// state in the others `Agent`s and incorporate that and handled changes in
    /// parameters coming from external inputs.
    fn update_based_on_shared_solution(&mut self, options: &Self::Options) -> Result<()>
    {
        self.algorithm_util_methods().load_shared_solution();

        let state_change = self.incorporate_shared_state()?;

        if state_change {
            self.calculate_objective_value(options)?;
            self.make_atomic_pointer_swap();
        }

        Ok(())
    }

    fn incorporate_shared_state(&mut self) -> Result<bool>;
}

pub trait AbLNSUtils
{
    type SolutionType: Solution + Debug + Clone;

    fn clone_algorithm_solution(&self) -> Self::SolutionType;

    fn load_shared_solution(&mut self);

    // You made the SolutionType to fix this issue. Now you are diviating
    // from it again. I think that this is the best approach
    fn update_objective_value(
        &mut self,
        objective_value: <Self::SolutionType as Solution>::ObjectiveValue,
    );

    fn swap_solution(&mut self, solution: Self::SolutionType);
}

#[allow(dead_code)]
pub enum ObjectiveValueType<O>
{
    Better(O),
    Worse,
    Force,
}

pub trait ObjectiveValue: Serialize {}
