pub mod operation_analytic;
pub mod operation_info;

use crate::models::work_order::operation::operation_info::OperationInfo;
use crate::models::{
    time_environment::day::Day, work_order::operation::operation_analytic::OperationAnalytic,
};

use crate::models::worker_environment::resources::Resources;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    activity: u32,
    resource: Resources,
    operation_info: OperationInfo,
    operation_analytic: OperationAnalytic,
    operation_dates: OperationDates,
}

impl Operation {
    pub fn new(
        activity: u32,
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

    pub fn activity(&self) -> u32 {
        self.activity
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
            "    Activity: {:>8}    |{:>11}|{:>14}|{:>8}|{:>6}|",
            self.activity,
            self.resource.to_string(),
            self.operation_info.work_remaining(),
            self.operation_analytic.duration(),
            self.operation_info.number(),
        )
    }
}

impl Operation {
    pub fn builder(activity: u32, resource: Resources, work_remaining: f64) -> OperationBuilder {
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
    activity: u32,
    resource: Resources,
    operation_info: OperationInfo,
    operation_analytic: OperationAnalytic,
    operation_dates: OperationDates,
}

impl OperationBuilder {
    fn with_activity(mut self, activity: u32) -> Self {
        self.activity = activity;
        self
    }

    fn with_resource(mut self, resource: Resources) -> Self {
        self.resource = resource;
        self
    }

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

    fn with_operation_analytic(mut self) -> Self {
        let operation_analytic = OperationAnalytic::new(2.0, 0);

        self.operation_analytic = operation_analytic;
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
