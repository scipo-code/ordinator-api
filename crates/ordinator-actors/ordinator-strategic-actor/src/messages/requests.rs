use core::fmt;
use std::collections::HashMap;
use std::fmt::Display;

use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "scheduler_message_type")]
pub struct StrategicTimeRequest
{
    pub periods: Vec<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StrategicPeriodsMessage
{
    pub period_lock: HashMap<String, bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestResource
{
    // SetResources {
    //     resources: Vec<Resources>,
    //     period_imperium: Period,
    //     capacity: f64,
    // },
    GetLoadings
    {
        periods_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities
    {
        periods_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetPercentageLoadings
    {
        periods_end: String,
        resources: Option<Vec<Resources>>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "scheduling_message_type")]
pub enum StrategicRequestScheduling
{
    Schedule(ScheduleChange),
    ExcludeFromPeriod(ScheduleChange),
}

impl StrategicRequestScheduling
{
    pub fn new_single_work_order(
        work_order_number: Vec<WorkOrderNumber>,
        period_string: String,
    ) -> Self
    {
        Self::Schedule(ScheduleChange {
            work_order_number,
            period_string,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleChange
{
    pub work_order_number: Vec<WorkOrderNumber>,
    pub period_string: String,
}

impl ScheduleChange
{
    pub fn new(work_order_number: Vec<WorkOrderNumber>, period_string: String) -> Self
    {
        Self {
            work_order_number,
            period_string,
        }
    }

    pub fn period_string(&self) -> String
    {
        self.period_string.clone()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicStatusMessage
{
    General,
    Period(String),
    WorkOrder(WorkOrderNumber),
}

impl StrategicStatusMessage
{
    pub fn new_period(period: String) -> Self
    {
        Self::Period(period)
    }
}

impl Display for StrategicStatusMessage
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            StrategicStatusMessage::General => write!(f, "general"),
            StrategicStatusMessage::Period(period) => write!(f, "period: {}", period),
            StrategicStatusMessage::WorkOrder(work_order_number) => {
                write!(f, "{:?}", work_order_number)
            }
        }
    }
}
