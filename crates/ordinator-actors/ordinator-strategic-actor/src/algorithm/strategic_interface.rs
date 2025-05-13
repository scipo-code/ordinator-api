use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;

use super::strategic_solution::StrategicSolution;

impl StrategicInterface for StrategicSolution
{
    // Double `Option` is not a good idea. I am not sure what the best approach is
    // forward here.
    fn scheduled_task(&self, work_order_number: &WorkOrderNumber) -> Option<&Option<Period>>
    {
        self.strategic_scheduled_work_orders.get(work_order_number)
    }

    fn supervisor_tasks(
        &self,
        periods: &[Period],
    ) -> std::collections::HashMap<WorkOrderNumber, Period>
    {
        self.strategic_scheduled_work_orders
            .clone()
            .into_iter()
            .filter_map(|(won, opt_str_per)| {
                opt_str_per.and_then(|per| periods.contains(&per).then_some((won, per)))
            })
            .collect()
    }

    fn all_scheduled_tasks(&self) -> std::collections::HashMap<WorkOrderNumber, Period>
    {
        self.strategic_scheduled_work_orders
            .clone()
            .into_iter()
            .filter_map(|(won, opt_per)| opt_per.map(|v| (won, v)))
            .collect()
    }
}
