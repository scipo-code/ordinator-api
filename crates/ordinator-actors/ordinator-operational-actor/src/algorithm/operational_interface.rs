use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;

use super::operational_solution::OperationalSolution;

impl OperationalInterface for OperationalSolution
{
    // Here you find the fruits of your labor. You should strive to make
    // this kind of code work.
    //
    // This is a filter on the `OperationalSolution` it is crucial to understand
    // this aspect to understand this.
    // QUESTION
    // What is it that this function is doing? What does it want to do?
    // I think that the approach here is to make something.
    // You want a function that takes a reference to a `operational_state_machine`
    // you could always make it as a m,
    // This was an ugly function when you created it and it is an ugly function now.
    // What should be done?
    // TODO [ ]
    // This is defined on a single operational_solution. Stick with
    // that! I do not think that there is a better way of doing things.
    fn marginal_fitness_for_operational_actor<'a>(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Vec<&'a MarginalFitness>
    {
        self.scheduled_work_order_activities
            .iter()
            .find(|woa_ass| woa_ass.0 == work_order_activity)
            .map(|woa_ass| &woa_ass.1.marginal_fitness)
            .iter()
            .collect::<Vec<_>>()
    }
}
