use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use tracing::{event};


use crate::agents::scheduler_agent::SchedulerAgent;

impl Display for SchedulerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "SchedulerAgent: \n
            Platform: {}, \n
            Manual Resources: {:?}, \n
            Backlog: {:?}, \n
            Scheduled Work Orders: {:?}, \n
            Periods: {:?}, \n
            WebSocketAgent Addr: {:?}", 
            self.platform, 
            self.scheduler_agent_algorithm.manual_resources_capacity, 
            self.scheduler_agent_algorithm.backlog.inner.len(), 
            self.scheduler_agent_algorithm.optimized_work_orders.inner.keys().collect::<Vec<_>>(), 
            self.scheduler_agent_algorithm.periods, 
            self.ws_agent_addr)
    }
}

pub struct DisplayableManualResource(pub HashMap<(String, String), f64, RandomState>);

impl fmt::Display for DisplayableManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ((resource, period), capacity) in self.0.iter() {
            writeln!(f, "Resource: {}, {}, Capacity: {}", resource, period, capacity)?;
        }
        Ok(())
    }
}

impl SchedulerAgent {
    pub fn log_optimized_work_orders(&self) {
        for (work_order_number, optimized) in &self.scheduler_agent_algorithm.optimized_work_orders.inner {
            
            match &optimized.locked_in_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = period.period_string)
                }
                None => event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "no locked period")
            }

            match &optimized.scheduled_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period.period_string)
                }
                None => event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "None")
            }

            for period in &optimized.excluded_from_periods {
                event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period)
            }
        }
    }
}