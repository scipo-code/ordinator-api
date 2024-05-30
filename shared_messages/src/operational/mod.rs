use actix::Message;
use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    agent_error::AgentError,
    models::worker_environment::availability::{Availability, TomlAvailability},
};

use self::{
    operational_request_resource::OperationalResourceRequest,
    operational_request_scheduling::OperationalSchedulingRequest,
    operational_request_status::OperationalStatusRequest,
    operational_request_time::OperationalTimeRequest,
    operational_response_resource::OperationalResourceResponse,
    operational_response_scheduling::OperationalSchedulingResponse,
    operational_response_status::OperationalStatusResponse,
    operational_response_time::OperationalTimeResponse,
};

pub mod operational_response_resource;
pub mod operational_response_scheduling;
pub mod operational_response_status;
pub mod operational_response_time;

pub mod operational_request_resource;
pub mod operational_request_scheduling;
pub mod operational_request_status;
pub mod operational_request_time;

pub enum OperationalRequestMessage {
    Status(OperationalStatusRequest),
    Scheduling(OperationalSchedulingRequest),
    Resource(OperationalResourceRequest),
    Time(OperationalTimeRequest),
}

impl Message for OperationalRequestMessage {
    type Result = Result<OperationalResponseMessage, AgentError>;
}
pub enum OperationalResponseMessage {
    Status(OperationalStatusResponse),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
}

#[derive(Serialize)]
pub enum OperationalResponse {
    Status,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationalConfiguration {
    pub availability: Availability,
    pub break_interval: TimeInterval,
    pub shift_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

impl OperationalConfiguration {
    pub fn new(
        availability: Availability,
        break_interval: TimeInterval,
        shift_interval: TimeInterval,
        toolbox_interval: TimeInterval,
    ) -> Self {
        Self {
            availability,
            break_interval,
            shift_interval,
            toolbox_interval,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlOperationalConfiguration {
    pub availability: TomlAvailability,
    pub break_interval: TomlTimeInterval,
    pub shift_interval: TomlTimeInterval,
    pub toolbox_interval: TomlTimeInterval,
}

impl From<TomlOperationalConfiguration> for OperationalConfiguration {
    fn from(value: TomlOperationalConfiguration) -> Self {
        OperationalConfiguration {
            availability: value.availability.into(),
            break_interval: value.break_interval.into(),
            shift_interval: value.shift_interval.into(),
            toolbox_interval: value.toolbox_interval.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeInterval {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomlTimeInterval {
    pub start: toml::value::Datetime,
    pub end: toml::value::Datetime,
}

impl From<TomlTimeInterval> for TimeInterval {
    fn from(value: TomlTimeInterval) -> Self {
        Self {
            start: NaiveTime::parse_from_str(&value.start.to_string(), "%H:%M:%S").unwrap(),
            end: NaiveTime::parse_from_str(&value.end.to_string(), "%H:%M:%S").unwrap(),
        }
    }
}

impl TimeInterval {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, date_time: DateTime<Utc>) -> bool {
        let time = date_time.time();

        if self.start <= time && time <= self.end {
            true
        } else {
            false
        }
    }
}
