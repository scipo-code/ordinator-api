pub mod operation_analytic;
pub mod operation_info;

use crate::scheduling_environment::work_order::operation::operation_info::OperationInfo;
use crate::scheduling_environment::{
    time_environment::day::Day, work_order::operation::operation_analytic::OperationAnalytic,
};

use crate::scheduling_environment::worker_environment::resources::Resources;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    pub activity: ActivityNumber,
    pub resource: Resources,
    pub operation_info: OperationInfo,
    pub operation_analytic: OperationAnalytic,
    pub operation_dates: OperationDates,
}

type Work = f64;

impl Operation {
    pub fn new(
        activity: ActivityNumber,
        resource: Resources,
        operation_info: OperationInfo,
        operation_analytic: OperationAnalytic,
        operation_dates: OperationDates,
    ) -> Self {
        Operation {
            activity,
            resource,
            operation_info,
            operation_analytic,
            operation_dates,
        }
    }

    pub fn work_remaining(&self) -> f64 {
        self.operation_info.work_remaining()
    }

    pub fn resource(&self) -> &Resources {
        &self.resource
    }

    pub fn number(&self) -> u32 {
        self.operation_info.number()
    }

    pub fn duration(&self) -> u32 {
        self.operation_analytic.duration()
    }

    pub fn operating_time(&self) -> f64 {
        self.operation_info.operating_time()
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct ActivityNumber(pub u32);

impl Serialize for ActivityNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for ActivityNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let activity_number_string = String::deserialize(deserializer).unwrap();
        let activity_number_primitive = activity_number_string.parse::<u32>().unwrap();

        Ok(ActivityNumber(activity_number_primitive))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationDates {
    possible_start: Day,
    target_finish: Day,
    earliest_start_datetime: DateTime<Utc>,
    earliest_finish_datetime: DateTime<Utc>,
}

impl OperationDates {
    pub fn new(
        possible_start: Day,
        target_finish: Day,
        earliest_start_datetime: DateTime<Utc>,
        earliest_finish_datetime: DateTime<Utc>,
    ) -> Self {
        assert!(possible_start < target_finish);
        OperationDates {
            possible_start,
            target_finish,
            earliest_start_datetime,
            earliest_finish_datetime,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    Activity: {:>8?}    |{:>11}|{:>14}|{:>8}|{:>6}|",
            self.activity,
            self.resource.to_string(),
            self.operation_info.work_remaining(),
            self.operation_analytic.duration(),
            self.operation_info.number(),
        )
    }
}

impl Operation {
    pub fn builder(
        activity: ActivityNumber,
        resource: Resources,
        work_remaining: Work,
    ) -> OperationBuilder {
        let operation_info = OperationInfo::new(1, work_remaining, 0.0, 0.0, 6.0);

        let operation_analytic = OperationAnalytic::new(1.0, 6);

        let operation_dates = OperationDates::new(
            Day::new(0, Utc::now()),
            Day::new(0, Utc::now()),
            Utc::now(),
            Utc::now(),
        );

        OperationBuilder {
            activity,
            resource,
            operation_info,
            operation_analytic,
            operation_dates,
        }
    }
}

pub struct OperationBuilder {
    activity: ActivityNumber,
    resource: Resources,
    operation_info: OperationInfo,
    operation_analytic: OperationAnalytic,
    operation_dates: OperationDates,
}

#[allow(dead_code)]
impl OperationBuilder {
    fn with_operation_info(
        mut self,
        number: u32,
        work_remaining: f64,
        work_performed: f64,
        work_adjusted: f64,
        operating_time: f64,
    ) -> Self {
        let operation_info = OperationInfo::new(
            number,
            work_remaining,
            work_performed,
            work_adjusted,
            operating_time,
        );

        self.operation_info = operation_info;
        self
    }

    fn with_operation_dates(mut self) -> Self {
        let operation_dates = OperationDates::new(
            Day::new(0, Utc::now()),
            Day::new(0, Utc::now()),
            Utc::now(),
            Utc::now(),
        );

        self.operation_dates = operation_dates;
        self
    }

    pub fn build(self) -> Operation {
        Operation {
            activity: self.activity,
            resource: self.resource,
            operation_info: self.operation_info,
            operation_analytic: self.operation_analytic,
            operation_dates: self.operation_dates,
        }
    }
}
