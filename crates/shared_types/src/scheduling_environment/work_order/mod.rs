pub mod display;
pub mod operation;
pub mod work_order_analytic;
pub mod work_order_dates;
pub mod work_order_info;

use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono::{Duration, NaiveDate};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::Asset;

use super::time_environment::period::Period;
use super::worker_environment::resources::Resources;

use self::operation::ActivityNumber;
use self::operation::Operation;
use self::operation::OperationBuilder;
use self::operation::Operations;
use self::operation::Work;
use self::work_order_analytic::status_codes::MaterialStatus;
use self::work_order_analytic::WorkOrderAnalytic;
use self::work_order_analytic::WorkOrderAnalyticBuilder;
use self::work_order_dates::WorkOrderDates;
use self::work_order_info::functional_location::FunctionalLocation;
use self::work_order_info::priority::Priority;
use self::work_order_info::work_order_type::WorkOrderType;
use self::work_order_info::WorkOrderInfo;
use self::work_order_info::WorkOrderInfoBuilder;

pub type WorkOrderValue = u64;

#[derive(Copy, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct WorkOrderNumber(pub u64);
impl WorkOrderNumber {
    pub fn is_dummy(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Debug for WorkOrderNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("WorkOrderNumber({})", self.0).bright_yellow()
        )
    }
}
// Everything in the `SchedulingEnvironment` should implement
// `Serialize` it has to, to be able to go into the database.
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkOrders {
    pub inner: HashMap<WorkOrderNumber, WorkOrder>,
    // Are these in the correct place in the code? Yes I think
    // that they are.
}

// WARN
// Configurations should only be used during initialization not the
// remaining parts of the code.
pub struct WorkOrdersBuilder {
    inner: Option<HashMap<WorkOrderNumber, WorkOrder>>,
}

impl WorkOrdersBuilder {
    pub fn build(self) -> WorkOrders {
        WorkOrders {
            inner: self.inner.unwrap_or_default(),
        }
    }

    pub fn work_order_builder<F>(&mut self, f: F, work_order_number: WorkOrderNumber) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderBuilder) -> &mut WorkOrderBuilder,
    {
        let mut work_order_builder = WorkOrder::builder(work_order_number);

        f(&mut work_order_builder);

        match &mut self.inner {
            Some(work_orders_inner) => {
                work_orders_inner.insert(
                    work_order_builder.work_order_number,
                    work_order_builder.build(),
                );
            }
            None => {
                let work_order_inner = HashMap::from([(
                    work_order_builder.work_order_number,
                    work_order_builder.build(),
                )]);

                self.inner = Some(work_order_inner);
            }
        }
        self
    }
}

impl WorkOrders {
    pub fn builder() -> WorkOrdersBuilder {
        WorkOrdersBuilder {
            inner: Some(HashMap::new()),
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder) {
        self.inner.insert(work_order.work_order_number, work_order);
    }

    pub fn new_work_order(&self, work_order_number: WorkOrderNumber) -> bool {
        !self.inner.contains_key(&work_order_number)
    }

    pub fn work_orders_by_asset(&self, asset: &Asset) -> HashMap<&WorkOrderNumber, &WorkOrder> {
        self.inner
            .iter()
            .filter(|(_, wo)| &wo.work_order_info.functional_location.asset == asset)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder {
    pub work_order_number: WorkOrderNumber,
    pub main_work_center: Resources,
    pub operations: Operations,
    pub work_order_analytic: WorkOrderAnalytic,
    pub work_order_dates: WorkOrderDates,
    pub work_order_info: WorkOrderInfo,
}

pub struct WorkOrderBuilder {
    work_order_number: WorkOrderNumber,
    main_work_center: Resources,
    operations: Operations,
    // FIX
    // Every operation needs to have a relation between them. There
    // is no way around this. It should be an enforced invariant.
    work_order_analytic: WorkOrderAnalytic,
    work_order_dates: WorkOrderDates,
    work_order_info: WorkOrderInfo,
}

impl WorkOrderBuilder {
    pub fn build(self) -> WorkOrder {
        WorkOrder {
            work_order_number: self.work_order_number,
            main_work_center: self.main_work_center,
            operations: self.operations,
            // TODO [ ]
            // Relations should be a function between the different on the `Operations` field.
            work_order_analytic: self.work_order_analytic,
            work_order_dates: self.work_order_dates,
            work_order_info: self.work_order_info,
        }
    }

    pub fn work_order_number(&mut self, work_order_number: WorkOrderNumber) -> &mut Self {
        self.work_order_number = work_order_number;
        self
    }

    pub fn main_work_center(mut self, main_work_center: Resources) -> Self {
        self.main_work_center = main_work_center;
        self
    }

    // TODO [ ]
    // Make this function simply reuse the functionality of the `Operations`.
    // QUESTION
    // How do we do this?
    // This is crucial! There is something that you do not understand here
    // How do we extract this so that it works?
    pub fn operations_builder<F>(
        &mut self,
        operation_number: u64,
        resource: Resources,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut OperationBuilder) -> &mut OperationBuilder,
    {
        let mut operations_builder = Operation::builder(operation_number, resource);

        f(&mut operations_builder);

        self.operations
            .0
            .insert(operation_number, operations_builder.build());

        self
    }

    pub fn work_order_analytic_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderAnalyticBuilder) -> &mut WorkOrderAnalyticBuilder,
    {
        let mut work_order_analytic_builder = WorkOrderAnalytic::builder();

        f(&mut work_order_analytic_builder);

        self.work_order_analytic = work_order_analytic_builder.build();
        self
    }

    pub fn work_order_info<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderInfoBuilder) -> &mut WorkOrderInfoBuilder,
    {
        let mut work_order_info_builder = WorkOrderInfo::builder();

        f(&mut work_order_info_builder);

        self.work_order_info = work_order_info_builder.build();
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityRelation {
    StartStart,
    FinishStart,
    Postpone(DateTime<Utc>),
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkOrderConfigurations {
    order_type_weights: HashMap<String, u64>,
    status_weights: HashMap<String, u64>,
    vis_priority_map: HashMap<char, u64>,
    wdf_priority_map: HashMap<u64, u64>,
    wgn_priority_map: HashMap<u64, u64>,
    wpm_priority_map: HashMap<char, u64>,
}

impl WorkOrderConfigurations {
    pub fn read_config() -> Result<Self> {
        let config_path = env::var("WORK_ORDER_WEIGHTINGS").expect("Work Order configuration parameters should always be provided through configuraion files specified in the .env file");
        let config_contents = fs::read_to_string(config_path).expect("Could not read config file");

        let config: WorkOrderConfigurations = serde_json::from_str(&config_contents)?;

        Ok(config)
    }
}

// You can remove all the initialization logic! So cool! I am not sure about the
//
// builder though.
// FIX [ ]
// Move all initialization into function calls.
// FIX [ ]
// Move the `latest_allowed_period` into a function as well.
impl WorkOrder {
    pub fn builder(work_order_number: WorkOrderNumber) -> WorkOrderBuilder {
        WorkOrderBuilder {
            work_order_number,
            main_work_center: todo!(),
            operations: todo!(),
            work_order_analytic: todo!(),
            work_order_dates: todo!(),
            work_order_info: todo!(),
        }
    }

    pub fn vendor(&self) -> bool {
        self.operations
            .0
            .values()
            .any(|opr| opr.resource.is_ven_variant())
    }

    pub fn work_order_value(
        &self,
        work_order_configurations: &WorkOrderConfigurations,
    ) -> WorkOrderValue {
        // FIX
        // This should be removed. Where should the global configs be read
        // from? I am not really sure. You have done a lot today! I think that
        // reading more. Is a really good idea. Maybe finish the one you started
        // quickly.
        // TODO [ ]
        // There can be no stray `configs` like these! They have to be handled
        // in a higher level.
        let base_value = match &self.work_order_info.work_order_type {
            WorkOrderType::Wdf(wdf_priority) => match wdf_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wdf_priority_map[int]
                        * work_order_configurations.order_type_weights["WDF"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                Priority::Int(int) if (&0..&8).contains(&int) => {
                    work_order_configurations.wgn_priority_map[int]
                        * work_order_configurations.order_type_weights["WGN"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    work_order_configurations.wpm_priority_map[char]
                        * work_order_configurations.order_type_weights["WPM"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wro(_) => todo!(),
            WorkOrderType::Other => work_order_configurations.order_type_weights["Other"],
        };

        let status_weight = {
            let mut work_order_value = 0;
            if self.work_order_analytic.user_status_codes.awsc {
                work_order_value += work_order_configurations.status_weights["AWSC"];
            }

            if self.work_order_analytic.user_status_codes.sece {
                work_order_value += work_order_configurations.status_weights["SECE"];
            }

            if self.work_order_analytic.system_status_codes.pcnf
                && self.work_order_analytic.system_status_codes.nmat
                || self.work_order_analytic.user_status_codes.smat
            {
                work_order_value += work_order_configurations.status_weights["PCNF_NMAT_SMAT"];
            }

            work_order_value
        };

        let total_weight = (base_value + status_weight)
            * self
                .work_order_load()
                .values()
                .map(|wor| wor.to_f64())
                .sum::<f64>() as u64;
        total_weight
    }

    pub fn work_order_load(&self) -> HashMap<Resources, Work> {
        self.operations
            .0
            .values()
            .fold(HashMap::default(), |mut acc, ele_opr| {
                *acc.entry(ele_opr.resource).or_insert(Work::from(0.0)) +=
                    ele_opr.operation_info.work_remaining;
                acc
            })
    }

    /// This method determines that earliest allow start date and period for the work order. This is
    /// a maximum of the material status and the earliest start period of the operations.
    /// TODO : A stance will have to be taken on the VEN, SHUTDOWN, and SUBNETWORKS.
    /// We will get an error here! The problem is that after this the EASD will not be contained
    /// anymore.
    // TODO [ ]
    // Extract these parameters into a config file.
    // TODO [ ]
    // Move this code into the Builder
    pub fn find_excluded_periods(&self, periods: &[Period]) -> HashSet<Period> {
        let mut excluded_periods: HashSet<Period> = HashSet::new();
        for (i, period) in periods.iter().enumerate() {
            if period < self.earliest_allowed_start_period(periods)
                || (self.vendor() && i <= 3)
                || (self.work_order_info.revision.shutdown() && i <= 3)
            {
                assert!(
                    self.earliest_allowed_start_period(&periods)
                        .end_date()
                        .date_naive()
                        >= self.work_order_dates.earliest_allowed_start_date
                );
                excluded_periods.insert(period.clone());
            }
        }
        excluded_periods
    }

    pub fn functional_location(&self) -> &FunctionalLocation {
        &self.work_order_info.functional_location
    }

    pub fn insert_operation(&mut self, operation: Operation) {
        self.operations.0.insert(operation.activity, operation);
    }

    // QUESTION
    // What should this function do? I think that the best approach is to
    // create something that will
    pub fn date_to_period<'a>(periods: &'a [Period], date_time: &NaiveDate) -> &'a Period {
        let period: Option<&Period> = periods.iter().find(|period| {
            period.start_date().date_naive() <= *date_time
                && period.end_date().date_naive() >= *date_time
        });

        // This is created in a horrible way. I think that the best approach here
        // is to make.
        // You are effectively making a new instance period which is not. Should
        // the work orders age? That is the fundamental question? I do not believe
        // so. One thing is for sure, the old period should be in the
        // `SchedulingEnvironment::time_environment`.
        match period {
            Some(period) => period,
            None => periods.first().unwrap(),
        }
    }

    // FIX
    // This should be based on a different formulation. This whole thing should be
    // formulated differently
    pub fn earliest_allowed_start_period<'a>(&'a self, periods: &'a [Period]) -> &'a Period {
        // This whole thing is bull shit.
        // TODO [ ]
        //

        let period =
            Self::date_to_period(periods, &self.work_order_dates.earliest_allowed_start_date);
        match &self.work_order_analytic.user_status_codes.clone().into() {
            MaterialStatus::Nmat => (&periods[0]).max(&period),
            MaterialStatus::Smat => (&periods[0]).max(&period),
            MaterialStatus::Cmat => (&periods[2]).max(&period),
            MaterialStatus::Pmat => (&periods[3]).max(&period),
            MaterialStatus::Wmat => (&periods[3]).max(&period),
            MaterialStatus::Unknown => panic!("WorkOrder does not have a material status"),
        }
    }

    pub fn latest_allowed_finish_period<'a>(&'a self, periods: &'a [Period]) -> &'a Period {
        todo!("Make the code for extracting this information")
    }

    pub fn unloading_point_contains_period(&self, clone: Period) -> bool {
        for operation in &self.operations.0 {
            if operation.1.unloading_point.period == Some(clone.clone()) {
                return true;
            }
        }
        false
    }

    pub fn unloading_point(&self) -> Option<Period> {
        self.operations
            .0
            .values()
            .find_map(|opr| opr.unloading_point.period.clone())
    }

    // fn random_latest_periods(&mut self, periods: &[Period]) {
    //     let mut rng = thread_rng();
    //     let random_period = periods.choose(&mut rng).unwrap();
    //     self.work_order_dates.latest_allowed_finish_period = random_period.clone();
    // }
}
impl FromStr for WorkOrderNumber {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = s.parse::<u64>()?;
        Ok(Self(number))
    }
}

pub type WorkOrderActivity = (WorkOrderNumber, ActivityNumber);

impl From<u64> for WorkOrderNumber {
    fn from(value: u64) -> Self {
        WorkOrderNumber(value)
    }
}

impl Serialize for WorkOrderNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkOrderNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let work_order_number_string = String::deserialize(deserializer).unwrap();
        let work_order_number_primitive = work_order_number_string.parse::<u64>().unwrap();
        Ok(WorkOrderNumber(work_order_number_primitive))
    }
}

// TODO [ ]
// This is a horrible practice! You should refactor it.
impl WorkOrder {
    pub fn work_order_test() -> Self {
        // The most important thing here is to make the construction as clean as possible.
        // QUESTION [ ]
        // How to best handle the `operation_analytic`?
        // I think we should make it seamless we that you cannot do it in the wrong way here.

        let work_order = WorkOrder::builder(WorkOrderNumber(2100000001))
            .main_work_center(Resources::MtnMech)
            // .operations_builder(10, Resources::Prodtech, |e| {
            //     e.operation_info(|oi| oi.number(1).work_remaining(10.0).operating_time(6.0))
            // })
            // .operations_builder(20, Resources::MtnMech, |ob| {
            //     ob.operation_info(|oi| oi.number(1).work_remaining(20.0).operating_time(6.0))
            // })
            // .operations_builder(20, Resources::MtnMech, |ob| {
            //     ob.operation_info(|oi| oi.number(1).work_remaining(30.0).operating_time(6.0))
            // })
            // .operations_builder(40, Resources::Prodtech, |ob| {
            //     ob.operation_info(|oi| oi.number(1).work_remaining(40.0).operating_time(6.0))
            // })
            // .work_order_analytic_builder(|woab| {
            //     woab.system_status_codes(|sta| sta.rel(true));
            //     woab.user_status_codes(|sta| sta.smat(true))
            // })
            // .work_order_info(|woib| {
            //     // FIX [ ]
            //     // This is wrong and it should be fixed. You should make the
            //     // code work correctly no matter what.
            //     woib.priority(Priority::Int(1));
            //     woib.work_order_type(WorkOrderType::Wdf(Priority::Int(1)))
            // })
            // .work_order_dates(|e| e.)
            .build();
        work_order
    }
}
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use crate::scheduling_environment::worker_environment::resources::Resources;

    use super::{
        operation::{ActivityNumber, OperationBuilder, Work},
        work_order_analytic::status_codes::{SystemStatusCodes, UserStatusCodes},
        work_order_dates::unloading_point::UnloadingPoint,
        work_order_dates::WorkOrderDates,
        work_order_info::functional_location::FunctionalLocation,
        work_order_info::priority::Priority,
        work_order_info::revision::Revision,
        work_order_info::system_condition::SystemCondition,
        work_order_info::work_order_text::WorkOrderText,
        work_order_info::work_order_type::WorkOrderType,
        work_order_info::WorkOrderInfoDetail,
        WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderNumber,
    };

    #[test]
    fn test_initialize_work_load() {
        let work_order = WorkOrder::work_order_test();

        assert_eq!(
            *work_order
                .work_order_load()
                .get(&Resources::from_str("PRODTECH").unwrap())
                .unwrap(),
            Work::from(50.0)
        );
        assert_eq!(
            *work_order
                .work_order_load()
                .get(&Resources::from_str("MTN-MECH").unwrap())
                .unwrap(),
            Work::from(50.0)
        );
    }
}
