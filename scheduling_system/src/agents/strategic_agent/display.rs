use std::fmt::Display;
use std::fmt::Write;

use crate::agents::strategic_agent::StrategicAgent;

impl Display for StrategicAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgent: \n
            Platform: {}, \n
            SchedulerAgentAlgorithm: {:?}, \n",
            self.asset, self.strategic_agent_algorithm,
        )
    }
}

impl StrategicAgent {
    pub fn format_selected_work_orders(
        &self,
        work_orders_number: Vec<u32>,
        period: Option<String>,
    ) -> String {
        let mut message = String::new();

        match period {
            Some(period) => writeln!(
                message,
                "Work orders scheduled for period: {} are: ",
                period,
            ),
            None => writeln!(message, "All work orders"),
        }
        .unwrap();

        writeln!(
            message,
            "                                      |EARL-PERIOD|AWSC|SECE|REVISION|TYPE|PRIO|VEN*| MAT|",
        )
        .unwrap();

        let mut work_orders = self
            .scheduling_environment
            .lock()
            .unwrap()
            .clone_work_orders();

        for work_order_number in work_orders_number {
            writeln!(
                message,
                "            Work order: {}    |{:>11}|{:<}|{:<}|{:>8}|{:>4}|{:>4}|{:>4}|{:?}|",
                work_order_number,
                work_orders
                    .inner
                    .get_mut(&work_order_number)
                    .unwrap()
                    .order_dates_mut()
                    .earliest_allowed_start_period
                    .period_string(),
                if work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .status_codes()
                    .awsc
                {
                    "AWSC"
                } else {
                    "----"
                },
                if work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .status_codes()
                    .sece
                {
                    "SECE"
                } else {
                    "----"
                },
                work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .revision()
                    .string,
                work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .order_type()
                    .get_type_string(),
                work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .priority()
                    .get_priority_string(),
                if work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .is_vendor()
                {
                    "VEN"
                } else {
                    "----"
                },
                work_orders
                    .inner
                    .get(&work_order_number)
                    .unwrap()
                    .status_codes()
                    .material_status
                    .to_string(),
            )
            .unwrap();
        }
        message
    }
}
