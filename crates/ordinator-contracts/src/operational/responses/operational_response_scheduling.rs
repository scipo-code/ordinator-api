use chrono::DateTime;
use chrono::Utc;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use serde::Serialize;

#[derive(Serialize)]
pub enum OperationalSchedulingResponse {
    EventList(Vec<ApiAssignmentEvents>),
}

#[derive(Serialize)]
pub struct ApiAssignmentEvents {
    event_info: EventInfo,
    json_assignments: Vec<ApiAssignment>,
}

impl ApiAssignmentEvents {
    pub fn new(event_info: EventInfo, json_assignments: Vec<ApiAssignment>) -> Self {
        Self {
            event_info,
            json_assignments,
        }
    }
}

#[derive(Serialize)]
pub struct ApiAssignment {
    event_type: EventType,
    start_date_time: DateTime<Utc>,
    finish_data_time: DateTime<Utc>,
}

impl ApiAssignment {
    pub fn new(
        event_type: EventType,
        start_date_time: DateTime<Utc>,
        finish_data_time: DateTime<Utc>,
    ) -> Self {
        Self {
            event_type,
            start_date_time,
            finish_data_time,
        }
    }
}

#[derive(Serialize)]
pub struct EventInfo {
    work_order_activity: Option<WorkOrderActivity>,
}

impl EventInfo {
    pub fn new(work_order_activity: Option<WorkOrderActivity>) -> Self {
        Self {
            work_order_activity,
        }
    }
}

#[derive(Serialize)]
pub enum EventType {
    WrenchTime,
    Break,
    Toolbox,
    OffShift,
    NonProductiveTime,
    Unavailable,
}
