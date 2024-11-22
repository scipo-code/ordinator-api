use std::fmt::Display;

use crate::agents::strategic_agent::StrategicAgent;

impl Display for StrategicAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgent: \n
            Platform: {}, \n",
            self.asset,
        )
    }
}
