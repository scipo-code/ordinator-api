use std::collections::BTreeMap;

use chrono::DateTime;
use chrono::Utc;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;

use super::tactical_solution::TacticalSolution;
use super::tactical_solution::TacticalWhereIsWorkOrder;

impl TacticalInterface for TacticalSolution
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&DateTime<Utc>, &DateTime<Utc>)>
    {
        let activities = self
            .tactical_work_orders
            .0
            .get(&work_order_activity.0)
            .unwrap();
        let scheduled_days = match &activities {
            ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Strategic => return None,
            ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Tactical(value) => {
                &value.0.get(&work_order_activity.1).unwrap().scheduled
            }
            ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => return None,
        };

        let start = scheduled_days.first().unwrap().0.date();
        let end = scheduled_days.last().unwrap().0.date();

        Some((start, end))
    }

    fn tactical_period(
        &self,
        work_order_number: &ordinator_scheduling_environment::work_order::WorkOrderNumber,
    ) -> Option<&ordinator_scheduling_environment::time_environment::period::Period>
    {
        todo!()
    }

    fn all_scheduled_tasks(
        &self,
    ) -> std::collections::HashMap<
        ordinator_scheduling_environment::work_order::WorkOrderNumber,
        std::collections::BTreeMap<
            ordinator_scheduling_environment::work_order::operation::ActivityNumber,
            ordinator_scheduling_environment::time_environment::day::Day,
        >,
    >
    {
        self
            // FIRST APPROACH
            .tactical_work_orders
            .0
            .iter()
            .clone()
            .map(|(won, whe_opt)| match whe_opt {
                ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Strategic => (won, None),
                ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Tactical(value) => {
                    (won, Some(value))
                }
                ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => (won, None),
            })
            .filter(|e| e.1.is_some())
            .map(|(won, e)| {
                (
                    *won,
                    e.unwrap()
                        .0
                        .clone()
                        .iter()
                        .map(|e| (*e.0, e.1.scheduled.first().as_ref().unwrap().0.clone()))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .collect()
    }
}
