use chrono::{DateTime, Utc};
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;

use super::tactical_solution::TacticalSolution;

impl TacticalInterface for TacticalSolution
{
    fn start_and_finish_dates(
            &self,
            work_order_activity: &WorkOrderActivity,
        ) -> Option<(&DateTime<Utc>, &DateTime<Utc>)> {
                let activities = self.tactical_work_orders.0.get(&work_order_activity.0).unwrap();
                let scheduled_days = &activities.0.get(&work_order_activity.1).unwrap().scheduled;

                let start = scheduled_days.first().unwrap().0.date();
                let end = scheduled_days.last().unwrap().0.date();

                (start, end)
        self.tactical_work_orders.0.get(&work_order_activity.0);
        todo!()
    }
}
