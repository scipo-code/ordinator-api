use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

// This should be a set of HTTP GET and POST endpoints. That is crucial to
// understand here. The goal here is to have an optimal backend data structure
// and then have a JSON api data structure. That is the best way of implementing
// this I do not see a different way.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalResourceRequest
{
    // SetResources(TacticalResources),
    GetLoadings
    {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetCapacities
    {
        days_end: String,
        select_resources: Option<Vec<Resources>>,
    },
    GetPercentageLoadings
    {
        days_end: String,
        resources: Option<Vec<Resources>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalSchedulingRequest
{
    Schedule(ScheduleChange),
    ScheduleMultiple(Vec<ScheduleChange>),
    ExcludeFromDay(ScheduleChange),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalStatusMessage
{
    General,
    Day(String),
}
#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum TacticalTimeRequest
{
    Days,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalUpdateRequest {}
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

pub enum TacticalRequestScheduling {}
pub enum TacticalRequestResource {}
pub enum TacticalSchedulingEnvironmentCommands {}
