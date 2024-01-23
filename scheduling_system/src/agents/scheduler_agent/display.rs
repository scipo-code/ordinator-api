use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use crate::agents::scheduler_agent::SchedulerAgent;

use super::scheduler_message::{ManualResource, SchedulerRequests, WorkOrderPeriodMapping};

impl Display for SchedulerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgent: \n
            Platform: {}, \n
            SchedulerAgentAlgorithm: {:?}, \n
            WebSocketAgent Addr: {:?}",
            self.platform, self.scheduler_agent_algorithm, self.ws_agent_addr
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

impl fmt::Display for SchedulerRequests {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            SchedulerRequests::Input(input) => {
                write!(f, "name: {}", input.name)?;

                for work_order_period_mapping in input.work_order_period_mappings.iter() {
                    writeln!(
                        f,
                        "work_order_period_mapping: {}",
                        work_order_period_mapping
                    )?;
                }
                for manual_resource in input.manual_resources.iter() {
                    writeln!(f, "manual_resource: {}", manual_resource)?;
                }
                Ok(())
            }
            SchedulerRequests::WorkPlanner(work_planner) => {
                write!(f, "work_planner: {:?}", work_planner.cannot_schedule)?;
                Ok(())
            }
            SchedulerRequests::Period(period) => {
                write!(f, "period: {:?}", period)?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for WorkOrderPeriodMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "work_order: {}, period: {:?}",
            self.work_order_number, self.period_status
        )
    }
}

impl fmt::Display for ManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "resource: {:?}, period: {}, capacity: {}",
            self.resource, self.period.period_string, self.capacity
        )
    }
}
