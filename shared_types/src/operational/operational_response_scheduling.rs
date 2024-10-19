use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{agent_error::AgentError, scheduling_environment::work_order::WorkOrderActivity};

#[derive(Serialize)]
pub enum OperationalSchedulingResponse {
    EventList(Vec<JsonAssignmentEvents>),
    Error(AgentError),
}

#[derive(Serialize)]
pub struct JsonAssignmentEvents {
    event_info: EventInfo,
    json_assignments: Vec<JsonAssignment>,
}

impl JsonAssignmentEvents {
    pub fn new(event_info: EventInfo, json_assignments: Vec<JsonAssignment>) -> Self {
        Self {
            event_info,
            json_assignments,
        }
    }
}

#[derive(Serialize)]
pub struct JsonAssignment {
    event_type: EventType,
    start_date_time: DateTime<Utc>,
    finish_data_time: DateTime<Utc>,
}

impl JsonAssignment {
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
