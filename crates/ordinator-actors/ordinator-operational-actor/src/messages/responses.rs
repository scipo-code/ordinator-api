use chrono::DateTime;
use chrono::Utc;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use serde::Serialize;

#[derive(Serialize)]
pub enum OperationalResourceResponse {}

#[derive(Serialize)]
pub enum OperationalSchedulingResponse
{
    EventList(Vec<ApiAssignmentEvents>),
}

// What should this be called? I think that the best word is to call it
// the
#[derive(Serialize)]
pub struct ApiAssignmentEvents
{
    event_info: EventInfo,
    json_assignments: Vec<ApiAssignment>,
}

impl ApiAssignmentEvents
{
    pub fn new(event_info: EventInfo, json_assignments: Vec<ApiAssignment>) -> Self
    {
        Self {
            event_info,
            json_assignments,
        }
    }
}

#[derive(Serialize)]
pub struct ApiAssignment
{
    event_type: EventType,
    start_date_time: DateTime<Utc>,
    finish_data_time: DateTime<Utc>,
}

impl ApiAssignment
{
    pub fn new(
        event_type: EventType,
        start_date_time: DateTime<Utc>,
        finish_data_time: DateTime<Utc>,
    ) -> Self
    {
        Self {
            event_type,
            start_date_time,
            finish_data_time,
        }
    }
}

#[derive(Serialize)]
pub struct EventInfo
{
    work_order_activity: Option<WorkOrderActivity>,
}

impl EventInfo
{
    pub fn new(work_order_activity: Option<WorkOrderActivity>) -> Self
    {
        Self {
            work_order_activity,
        }
    }
}

#[derive(Serialize)]
pub enum EventType
{
    WrenchTime,
    Break,
    Toolbox,
    OffShift,
    NonProductiveTime,
    Unavailable,
}
use ordinator_scheduling_environment::worker_environment::resources::Id;

#[derive(Serialize)]
pub struct OperationalResponseStatus
{
    id: Id,
    assign_number_of_activities: u64,
    assess_number_of_activities: u64,
    unassign_number_of_activities: u64,
    objective: OperationalObjectiveValue,
}

impl OperationalResponseStatus
{
    pub fn new(
        id: Id,
        assign_number_of_activities: u64,
        assess_number_of_activities: u64,
        unassign_number_of_activities: u64,
        objective: OperationalObjectiveValue,
    ) -> Self
    {
        Self {
            id,
            assign_number_of_activities,
            assess_number_of_activities,
            unassign_number_of_activities,
            objective,
        }
    }
}

use crate::algorithm::operational_solution::OperationalObjectiveValue;

#[derive(Serialize)]
pub enum OperationalTimeResponse {}
