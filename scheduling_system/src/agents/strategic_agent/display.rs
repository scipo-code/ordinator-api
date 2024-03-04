use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use crate::agents::strategic_agent::StrategicAgent;

impl Display for StrategicAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgent: \n
            Platform: {}, \n
            SchedulerAgentAlgorithm: {:?}, \n
            WebSocketAgent Addr: {:?}",
            self.platform, self.scheduler_agent_algorithm, self.orchestrator_agent_addr
        )
    }
}

pub struct DisplayableManualResource(pub HashMap<(String, String), f64, RandomState>);

impl fmt::Display for DisplayableManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ((resource, period), capacity) in self.0.iter() {
            writeln!(
                f,
                "Resource: {}, {}, Capacity: {}",
                resource, period, capacity
            )?;
        }
        Ok(())
    }
}
