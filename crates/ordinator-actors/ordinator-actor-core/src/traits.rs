use std::fmt::Debug;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

pub type ActorLinkToSchedulingEnvironment<'a> = MutexGuard<'a, SchedulingEnvironment>;

pub trait ActorBasedLargeNeighborhoodSearch {
    type Algorithm: AbLNSUtils;
    type Options;

    // This should be changed as well. You do not want to lock the database on every retry...
    // Ahh this is why you made the SystemConfiguration use the ArcSwap... Hmm... This is
    // really annoying. I think that the best approach. You cannot lock the Scheduling
    // environment on every iteration. I think that you should... The value of having
    // the Configuration is the database outweights the downside here.
    //
    // This is the issue. You have to work on getting the code into the correct
    // form here. You should decide whether you should work on getting the code
    // to work with. I believe that the user should be able to change these things.
    //
    //
    fn run_lns_iteration(
        &mut self,
        configurations: ActorLinkToSchedulingEnvironment,
        id: &Id,
    ) -> Result<()> {
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

    // So this should be gotten from the SchedulingEnvironment.
    fn derive_options(configurations: &ActorLinkToSchedulingEnvironment, id: &Id) -> Self::Options;

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm;

    fn make_atomic_pointer_swap(&mut self);

    // So the issue ultimately arises due to you wanting to avoid a state
    // change when the options for the strategic actor updates itself.
    //
    // You will have to get over this as quickly as possible. I think
    // that maybe the best approach here will be to
    //
    // I think that dependency injecting the Option is a fine approach
    // the issue arises when you have to upsteam also lock the scheduling
    // environment. Yes that is the issue. I think that this should simply
    // be apart of the `StateLink`.
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
    fn update_based_on_shared_solution(&mut self, options: &Self::Options) -> Result<()> {
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

pub trait AbLNSUtils {
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
pub enum ObjectiveValueType<O> {
    Better(O),
    Worse,
    Force,
}

pub trait ObjectiveValue: Serialize {}
