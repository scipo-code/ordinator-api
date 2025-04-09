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
        self.strategic_scheduled_work_orders.get(&work_order_number)
    }
}
