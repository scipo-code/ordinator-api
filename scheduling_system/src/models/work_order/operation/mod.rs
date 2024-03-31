pub mod operation_analytic;
pub mod operation_info;

use crate::models::work_order::operation::operation_analytic::OperationAnalytic;
use crate::models::work_order::operation::operation_info::OperationInfo;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_messages::resources::Resources;
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
    possible_start: DateTime<Utc>,
    target_finish: DateTime<Utc>,
    earliest_start_datetime: DateTime<Utc>,
    earliest_finish_datetime: DateTime<Utc>,
}

impl OperationDates {
    pub fn new(
        possible_start: DateTime<Utc>,
        target_finish: DateTime<Utc>,
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

#[cfg(test)]
mod tests {
    use super::{Operation, OperationAnalytic, OperationDates, OperationInfo};
    use chrono::Utc;
    use shared_messages::resources::Resources;

    impl Operation {
        pub fn new_test(activity: u32, work_center: Resources, work_remaining: f64) -> Self {
            let operation_info = OperationInfo::new(1, work_remaining, 0.0, 0.0, 6.0);

            let operation_analytic = OperationAnalytic::new(1.0, 6);

            let operation_dates =
                OperationDates::new(Utc::now(), Utc::now(), Utc::now(), Utc::now());

            Operation {
                activity,
                resource: work_center,
                operation_info,
                operation_analytic,
                operation_dates,
            }
        }
    }
}
