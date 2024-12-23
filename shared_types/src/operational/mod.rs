use actix::Message;
use chrono::{DateTime, NaiveTime, TimeDelta, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::{
    scheduling_environment::worker_environment::{availability::Availability, resources::Id},
    Asset, ConstraintState,
};
use anyhow::Result;

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
pub enum OperationalRequest {
    GetIds(Asset),
    AllOperationalStatus(Asset),
    ForOperationalAgent((Asset, String, OperationalRequestMessage)),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalRequestMessage {
    Status(OperationalStatusRequest),
    Scheduling(OperationalSchedulingRequest),
    Resource(OperationalResourceRequest),
    Time(OperationalTimeRequest),
}

impl Message for OperationalRequestMessage {
    type Result = Result<OperationalResponseMessage>;
}

#[derive(Serialize)]
pub enum OperationalResponseMessage {
    Status(OperationalStatusResponse),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
}

#[derive(Serialize)]
pub struct OperationalStatus {
    objective: f64,
}

#[derive(Serialize)]
pub enum OperationalResponse {
    AllOperationalStatus(Vec<OperationalResponseMessage>),
    OperationalIds(Vec<Id>),
    OperationalState(OperationalResponseMessage),
    NoOperationalAgentFound(String),
}

#[derive(Deserialize, Debug, Serialize, Clone)]
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct TimeInterval {
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub start: NaiveTime,
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub end: NaiveTime,
}

fn deserialize_time_interval<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time_str: String = Deserialize::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time_str, "%H:%M:%S").map_err(de::Error::custom)
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

    pub fn from_date_times(
        start_date_time: DateTime<Utc>,
        finish_date_time: DateTime<Utc>,
    ) -> Self {
        Self {
            start: start_date_time.time(),
            end: finish_date_time.time(),
        }
    }

    pub fn contains(&self, date_time: &DateTime<Utc>) -> bool {
        let time = date_time.time();

        if self.start > self.end {
            (self.start <= time && time <= NaiveTime::from_hms_opt(23, 59, 59).unwrap())
                || (NaiveTime::from_hms_opt(0, 0, 0).unwrap() <= time && time < self.end)
        } else {
            self.start <= time && time < self.end
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
    fn test_time_interval_contains_1() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T00:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_2() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T20:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_contains_3() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_4() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }

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
