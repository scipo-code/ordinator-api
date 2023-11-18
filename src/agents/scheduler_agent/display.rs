use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;

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

