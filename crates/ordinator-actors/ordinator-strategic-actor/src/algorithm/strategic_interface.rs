use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;

impl StrategicInterface for StrategicSolution
{
    fn scheduled_task(&self, work_order_number: &WorkOrderNumber) -> Option<Option<Period>>
    {
        self.strategic_scheduled_work_orders
            .get(&work_order_activity.0)
    }
}
