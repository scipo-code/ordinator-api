use std::collections::HashMap;

use actix::Message;
use chrono::{DateTime, NaiveTime, TimeDelta, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    agent_error::AgentError,
    models::worker_environment::{
        availability::{Availability, TomlAvailability},
        resources::Id,
    },
    AlgorithmState, ConstraintState,
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

type OperationalId = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct OperationalRequest {
    pub operational_target: OperationalTarget,
    pub operational_request_message: OperationalRequestMessage,
}

impl OperationalRequest {
    pub fn new(
        operational_target: OperationalTarget,
        operational_request_message: OperationalRequestMessage,
    ) -> Self {
        Self {
            operational_target,
            operational_request_message,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalRequestMessage {
    Status(OperationalStatusRequest),
    Scheduling(OperationalSchedulingRequest),
    Resource(OperationalResourceRequest),
    Time(OperationalTimeRequest),
    Test,
}

impl Message for OperationalRequestMessage {
    type Result = Result<OperationalResponseMessage, AgentError>;
}

#[derive(Serialize)]
pub enum OperationalResponseMessage {
    Status(OperationalStatusResponse),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
    Test(AlgorithmState<OperationalInfeasibleCases>),
}

#[derive(Serialize)]
pub struct OperationalResponse {
    id: OperationalTarget,
    operational_response_message: Vec<OperationalResponseMessage>,
}

impl OperationalResponse {
    pub fn new(
        id: OperationalTarget,
        operational_response_message: Vec<OperationalResponseMessage>,
    ) -> Self {
        Self {
            id,
            operational_response_message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationalConfiguration {
    pub availability: Availability,
    pub break_interval: TimeInterval,
    pub off_shift_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

impl OperationalConfiguration {
    pub fn new(
        availability: Availability,
        break_interval: TimeInterval,
        off_shift_interval: TimeInterval,
        toolbox_interval: TimeInterval,
    ) -> Self {
        Self {
            availability,
            break_interval,
            off_shift_interval,
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
        let shift_interval: TimeInterval = value.shift_interval.into();
        let off_shift_interval: TimeInterval = shift_interval.invert();
        OperationalConfiguration {
            availability: value.availability.into(),
            break_interval: value.break_interval.into(),
            off_shift_interval,
            toolbox_interval: value.toolbox_interval.into(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct TimeInterval {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl Default for TimeInterval {
    fn default() -> Self {
        Self {
            start: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        }
    }
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
        assert_ne!(start, end);
        Self { start, end }
    }

    pub fn contains(&self, date_time: &DateTime<Utc>) -> bool {
        let time = date_time.time();

        if self.start <= time && time < self.end {
            true
        } else {
            false
        }
    }

    pub fn duration(&self) -> TimeDelta {
        if self.end < self.start {
            TimeDelta::new(86400, 0).unwrap() - (self.end - self.start).abs()
        } else {
            (self.end - self.start).abs()
        }
    }

    pub fn invert(&self) -> TimeInterval {
        let inverted_start = self.end;
        let inverted_end = self.start;

        let inverted_time_interval = TimeInterval {
            start: inverted_start,
            end: inverted_end,
        };
        assert_eq!(self.duration(), inverted_time_interval.duration());
        inverted_time_interval
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, clap::ValueEnum)]
pub enum OperationalTarget {
    #[clap(skip)]
    Single(OperationalId),
    All,
}

#[derive(Serialize)]
pub struct OperationalInfeasibleCases {
    pub operation_overlap: ConstraintState<String>,
}

impl OperationalInfeasibleCases {
    pub fn all_feasible(&self) -> bool {
        if self.operation_overlap != ConstraintState::Feasible {
            return false;
        }
        true
    }
}

impl Default for OperationalInfeasibleCases {
    fn default() -> Self {
        Self {
            operation_overlap: ConstraintState::Undetermined,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_time_interval_duration() {
        let start = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(12 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(2, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(5 * 3600, 0).unwrap()
        );
        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(1, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(2 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(time_interval.duration(), TimeDelta::new(0, 0).unwrap());
    }
}
