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
            self.manual_resources, 
            self.backlog.inner.len(), 
            self.scheduled_work_orders, 
            self.periods, 
            self.ws_agent_addr)
    }
}