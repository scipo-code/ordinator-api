use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize};

use super::TimePeriod;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "scheduling_message_type")]
pub enum StrategicSchedulingMessage {
    Schedule(SingleWorkOrder),
    ScheduleMultiple(Vec<SingleWorkOrder>),
    ExcludeFromPeriod(SingleWorkOrder),
}

impl StrategicSchedulingMessage {
    pub fn new_single_work_order(work_order_number: u32, period_string: String) -> Self {
        Self::Schedule(SingleWorkOrder {
            work_order_number,
            period_string,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleWorkOrder {
    work_order_number: u32,
    period_string: String,
}

impl SingleWorkOrder {
    pub fn new(work_order_number: u32, period_string: String) -> Self {
        Self {
            work_order_number,
            period_string,
        }
    }

    pub fn get_work_order_number(&self) -> u32 {
        self.work_order_number
    }

    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: u32,
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

impl StrategicSchedulingMessage {
    pub fn new_schedule_test() -> Self {
        let schedule_single_work_order =
            SingleWorkOrder::new(2200002020, "2023-W47-48".to_string());
        Self::Schedule(schedule_single_work_order)
    }
}
