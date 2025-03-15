use std::fmt::Display;

use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::operation::{
    ActivityNumber, Work, operation_info::NumberOfPeople,
};
use ordinator_scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct TacticalObjectiveValue {
    pub objective_value: u64,
    pub urgency: (u64, u64),
    pub resource_penalty: (u64, u64),
}

impl Default for TacticalObjectiveValue {
    fn default() -> Self {
        Self {
            objective_value: u64::MAX,
            urgency: (u64::MAX, u64::MAX),
            resource_penalty: (u64::MAX, u64::MAX),
        }
    }
}
impl TacticalObjectiveValue {
    pub fn new(objective_value: u64, urgency: (u64, u64), resource_penalty: (u64, u64)) -> Self {
        Self {
            objective_value,
            urgency,
            resource_penalty,
        }
    }

    pub fn aggregate_objectives(&mut self) {
        self.objective_value =
            self.urgency.0 * self.urgency.1 + self.resource_penalty.0 * self.resource_penalty.1;
    }
}

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize)]
pub struct OperationSolution {
    pub scheduled: Vec<(Day, Work)>,
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_order_activity: WorkOrderActivity,
}

impl OperationSolution {
    pub fn new(
        scheduled: Vec<(Day, Work)>,
        resource: Resources,
        number: NumberOfPeople,
        work_remaining: Work,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> OperationSolution {
        OperationSolution {
            scheduled,
            resource,
            number,
            work_remaining,
            work_order_activity: (work_order_number, activity_number),
        }
    }
}

impl Display for OperationSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.work_order_activity)?;
        for scheduled in &self.scheduled {
            write!(f, "{} on {}", scheduled.1, scheduled.0)?
        }
        Ok(())
    }
}
