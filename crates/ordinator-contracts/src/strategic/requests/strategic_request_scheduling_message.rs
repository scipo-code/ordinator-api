use std::collections::HashSet;

use clap::Args;
use serde::{Deserialize, Deserializer, Serialize};

use crate::scheduling_environment::work_order::WorkOrderNumber;

use crate::agents::strategic::TimePeriod;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "scheduling_message_type")]
pub enum StrategicRequestScheduling {
    Schedule(ScheduleChange),
    ExcludeFromPeriod(ScheduleChange),
}

impl StrategicRequestScheduling {
    pub fn new_single_work_order(
        work_order_number: Vec<WorkOrderNumber>,
        period_string: String,
    ) -> Self {
        Self::Schedule(ScheduleChange {
            work_order_number,
            period_string,
        })
    }
}

#[derive(Args, Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleChange {
    pub work_order_number: Vec<WorkOrderNumber>,
    pub period_string: String,
}

impl ScheduleChange {
    pub fn new(work_order_number: Vec<WorkOrderNumber>, period_string: String) -> Self {
        Self {
            work_order_number,
            period_string,
        }
    }

    pub fn period_string(&self) -> String {
        self.period_string.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: WorkOrderNumber,
    pub period_status: WorkOrderStatusInPeriod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderStatusInPeriod {
    pub locked_in_period: Option<TimePeriod>,
    #[serde(deserialize_with = "deserialize_period_set")]
    pub excluded_from_periods: HashSet<TimePeriod>,
}

fn deserialize_period_set<'de, D>(deserializer: D) -> Result<HashSet<TimePeriod>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TimePeriod>::deserialize(deserializer)?;
    let mut set = HashSet::new();
    for time_period_map in vec {
        set.insert(TimePeriod {
            period_string: time_period_map.period_string,
        });
    }
    Ok(set)
}
