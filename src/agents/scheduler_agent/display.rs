use crate::agents::scheduler_agent::SchedulerAgent;
use std::fmt::Display;


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
            self.scheduler_agent_algorithm.scheduled_work_orders, 
            self.scheduler_agent_algorithm.periods, 
            self.ws_agent_addr)
    }
}