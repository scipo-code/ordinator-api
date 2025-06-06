pub mod display;
pub mod operation;
pub mod work_order_analytic;
pub mod work_order_dates;
pub mod work_order_info;

use std::collections::HashMap;
use std::collections::HashSet;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::TimeDelta;
use colored::Colorize;
use serde::Deserialize;
use serde::Serialize;
use work_order_dates::WorkOrderDatesBuilder;

use self::operation::ActivityNumber;
use self::operation::Operation;
use self::operation::OperationBuilder;
use self::operation::Operations;
use self::operation::Work;
use self::work_order_analytic::WorkOrderAnalytic;
use self::work_order_analytic::WorkOrderAnalyticBuilder;
use self::work_order_analytic::status_codes::MaterialStatus;
use self::work_order_dates::WorkOrderDates;
use self::work_order_info::WorkOrderInfo;
use self::work_order_info::WorkOrderInfoBuilder;
use self::work_order_info::functional_location::FunctionalLocation;
use self::work_order_info::priority::Priority;
use self::work_order_info::work_order_type::WorkOrderType;
use super::time_environment::period::Period;
use super::worker_environment::resources::Resources;
use crate::Asset;
use crate::time_environment::MaterialToPeriod;

pub type WorkOrderValue = u64;

#[derive(Copy, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct WorkOrderNumber(pub u64);
impl WorkOrderNumber
{
    pub fn is_dummy(&self) -> bool
    {
        self.0 == 0
    }
}

impl std::fmt::Debug for WorkOrderNumber
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{}",
            format!("WorkOrderNumber({})", self.0).bright_yellow()
        )
    }
}
// Everything in the `SchedulingEnvironment` should implement
// `Serialize` it has to, to be able to go into the database.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrders
{
    pub inner: HashMap<WorkOrderNumber, WorkOrder>,
    // Are these in the correct place in the code? Yes I think
    // that they are.
}

// WARN
// Configurations should only be used during initialization not the
// remaining parts of the code.
pub struct WorkOrdersBuilder
{
    inner: Option<HashMap<WorkOrderNumber, WorkOrder>>,
}

impl WorkOrdersBuilder
{
    pub fn build(self) -> WorkOrders
    {
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

impl WorkOrders
{
    pub fn builder() -> WorkOrdersBuilder
    {
        WorkOrdersBuilder {
            inner: Some(HashMap::new()),
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder)
    {
        self.inner.insert(work_order.work_order_number, work_order);
    }

    pub fn new_work_order(&self, work_order_number: WorkOrderNumber) -> bool
    {
        !self.inner.contains_key(&work_order_number)
    }

    pub fn work_orders_by_asset(&self, asset: &Asset) -> HashMap<&WorkOrderNumber, &WorkOrder>
    {
        self.inner
            .iter()
            .filter(|(_, wo)| &wo.work_order_info.functional_location.asset == asset)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder
{
    pub work_order_number: WorkOrderNumber,
    pub main_work_center: Resources,
    pub operations: Operations,
    pub work_order_analytic: WorkOrderAnalytic,
    pub work_order_dates: WorkOrderDates,
    pub work_order_info: WorkOrderInfo,
}

pub struct WorkOrderBuilder
{
    work_order_number: WorkOrderNumber,
    main_work_center: Option<Resources>,
    operations: Operations,
    // FIX
    // Every operation needs to have a relation between them. There
    // is no way around this. It should be an enforced invariant.
    work_order_analytic: Option<WorkOrderAnalytic>,
    work_order_dates: Option<WorkOrderDates>,
    work_order_info: Option<WorkOrderInfo>,
}

impl WorkOrderBuilder
{
    pub fn build(self) -> WorkOrder
    {
        WorkOrder {
            work_order_number: self.work_order_number,
            main_work_center: self
                .main_work_center
                .expect("Missing field initializations on the WorkOrderBuilder"),
            operations: self.operations,
            work_order_analytic: self
                .work_order_analytic
                .expect("Missing field initializations on the WorkOrderBuilder"),
            work_order_dates: self
                .work_order_dates
                .expect("Missing field initializations on the WorkOrderBuilder"),
            work_order_info: self
                .work_order_info
                .expect("Missing field initializations on the WorkOrderBuilder"),
        }
    }

    pub fn work_order_number(&mut self, work_order_number: WorkOrderNumber) -> &mut Self
    {
        self.work_order_number = work_order_number;
        self
    }

    pub fn main_work_center(mut self, main_work_center: Resources) -> Self
    {
        self.main_work_center = Some(main_work_center);
        self
    }

    // TODO [ ]
    // Make this function simply reuse the functionality of the `Operations`.
    // QUESTION
    // How do we do this?
    // This is crucial! There is something that you do not understand here
    // How do we extract this so that it works?
    pub fn operations_builder<F>(mut self, operation_number: u64, resource: Resources, f: F) -> Self
    where
        F: FnOnce(OperationBuilder) -> OperationBuilder,
    {
        let operations_builder = Operation::builder(operation_number, resource);

        let operations_builder = f(operations_builder);

        self.operations
            .0
            .insert(operation_number, operations_builder.build());

        self
    }

    pub fn operations(mut self, operations: Operations) -> Self
    {
        self.operations = operations;
        self
    }

    pub fn work_order_analytic_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderAnalyticBuilder) -> WorkOrderAnalyticBuilder,
    {
        let work_order_analytic_builder = WorkOrderAnalytic::builder();

        let work_order_analytic_builder = configure(work_order_analytic_builder);

        self.work_order_analytic = Some(work_order_analytic_builder.build());
        self
    }

    pub fn work_order_info_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderInfoBuilder) -> WorkOrderInfoBuilder,
    {
        let work_order_info_builder = WorkOrderInfo::builder();

        let work_order_info_builder = configure(work_order_info_builder);

        self.work_order_info = Some(work_order_info_builder.build());
        self
    }

    pub fn work_order_dates_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderDatesBuilder) -> WorkOrderDatesBuilder,
    {
        let work_order_dates_builder = WorkOrderDates::builder();

        let work_order_dates_builder = configure(work_order_dates_builder);

        self.work_order_dates = Some(work_order_dates_builder.build());
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityRelation
{
    StartStart,
    FinishStart,
    Postpone(TimeDelta),
}

// `operating_time` is separate from the work order data and should be removed
// from the `scheduling_environment`
#[allow(dead_code)]
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderConfigurations
{
    pub order_type_weights: HashMap<String, u64>,
    pub status_weights: HashMap<String, u64>,
    pub vis_priority_map: HashMap<char, u64>,
    pub wdf_priority_map: HashMap<String, u64>,
    pub wgn_priority_map: HashMap<String, u64>,
    pub wpm_priority_map: HashMap<char, u64>,
    pub clustering_weights: ClusteringWeights,
    pub operating_time: u64,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct ClusteringWeights
{
    pub asset: u64,
    pub sector: u64,
    pub system: u64,
    pub subsystem: u64,
    pub equipment_tag: u64,
}

// You can remove all the initialization logic! So cool! I am not sure about the
//
// builder though.
// FIX [ ]
// Move all initialization into function calls.
// FIX [ ]
// Move the `latest_allowed_period` into a function as well.
impl WorkOrder
{
    pub fn builder(work_order_number: WorkOrderNumber) -> WorkOrderBuilder
    {
        WorkOrderBuilder {
            work_order_number,
            main_work_center: None,
            operations: Operations::default(),
            work_order_analytic: None,
            work_order_dates: None,
            work_order_info: None,
        }
    }

    pub fn vendor(&self) -> bool
    {
        self.operations
            .0
            .values()
            .any(|opr| opr.resource.is_ven_variant())
    }

    pub fn work_order_value(
        &self,
        work_order_configurations: &WorkOrderConfigurations,
    ) -> Result<WorkOrderValue>
    {
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
                    work_order_configurations.wdf_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WDF"]
                }
                _ => bail!("Received a wrong input number work order priority: {wdf_priority:#?}"),
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wgn_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WGN"]
                }
                _ => bail!("Received a wrong input number work order priority: {wgn_priority:#?}"),
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    work_order_configurations.wpm_priority_map[char]
                        * work_order_configurations.order_type_weights["WPM"]
                }
                _ => bail!("Received a wrong input number work order priority: {wpm_priority:#?}"),
            },
            // ISSUE #000 handle-the-wro-work-order.
            WorkOrderType::Wro(wro_priority) => match wro_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wgn_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WGN"]
                }
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    work_order_configurations.wpm_priority_map[char]
                        * work_order_configurations.order_type_weights["WPM"]
                }
                _ => bail!("Received a wrong input number work order priority: {wro_priority:#?}"),
            },
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

        let weight = (base_value + status_weight)
            * (self
                .work_order_load()
                .context("An error occured line creating the work_order_load")?
                .values()
                .map(|wor| wor.to_f64())
                .sum::<f64>() as u64);
        Ok(weight)
    }

    pub fn work_order_load(&self) -> Result<HashMap<Resources, Work>>
    {
        self.operations
            .0
            .values()
            .try_fold(HashMap::default(), |mut acc, ele_opr: &Operation| {
                ensure!(
                    ele_opr.operation_info.work_remaining >= Work::from(0.0),
                    "{:#?}",
                    ele_opr
                );
                *acc.entry(ele_opr.resource).or_insert(Work::from(0.0)) +=
                    ele_opr.operation_info.work_remaining.round();
                Ok(acc)
            })
    }

    /// This method determines that earliest allow start date and period for the
    /// work order. This is a maximum of the material status and the
    /// earliest start period of the operations. TODO : A stance will have
    /// to be taken on the VEN, SHUTDOWN, and SUBNETWORKS. We will get an
    /// error here! The problem is that after this the EASD will not be
    /// contained anymore.
    // TODO [ ]
    // Extract these parameters into a config file.
    // TODO [ ]
    // Move this code into the Builder
    pub fn find_excluded_periods(
        &self,
        periods: &[Period],
        material_to_periods: &MaterialToPeriod,
    ) -> HashSet<Period>
    {
        periods
            .iter()
            .enumerate()
            .filter(|(i, per)| {
                *per < self.earliest_allowed_start_period(periods, material_to_periods)
                    || (self.vendor() && *i <= 3)
                    || (self.work_order_info.revision.shutdown() && *i <= 3)
            })
            .map(|(_, per)| per.clone())
            .collect()
    }

    pub fn functional_location(&self) -> &FunctionalLocation
    {
        &self.work_order_info.functional_location
    }

    pub fn insert_operation(&mut self, operation: Operation)
    {
        self.operations.0.insert(operation.activity, operation);
    }

    // QUESTION
    // What should this function do? I think that the best approach is to
    // create something that will
    pub fn date_to_period<'a>(periods: &'a [Period], date_time: &NaiveDate) -> &'a Period
    {
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
    pub fn earliest_allowed_start_period<'a>(
        &'a self,
        periods: &'a [Period],
        material_to_periods: &MaterialToPeriod,
    ) -> &'a Period
    {
        // This whole thing is bull shit.
        // TODO [ ]
        //

        // This is also not this straihtforward. There is an interplay between
        // this and the
        // assert!(
        //     self.earliest_allowed_start_period(&periods)
        //         .end_date()
        //         .date_naive()
        //         >= self.work_order_dates.earliest_allowed_start_date
        // );
        let period =
            Self::date_to_period(periods, &self.work_order_dates.earliest_allowed_start_date);
        match &self.work_order_analytic.user_status_codes.clone().into() {
            MaterialStatus::Nmat => (&periods[material_to_periods.nmat]).max(period),
            MaterialStatus::Smat => (&periods[material_to_periods.smat]).max(period),
            MaterialStatus::Cmat => (&periods[material_to_periods.cmat]).max(period),
            MaterialStatus::Pmat => (&periods[material_to_periods.pmat]).max(period),
            MaterialStatus::Wmat => (&periods[material_to_periods.wmat]).max(period),
            MaterialStatus::Unknown => panic!("WorkOrder does not have a material status"),
        }
    }

    pub fn latest_allowed_finish_period<'a>(&'a self, periods: &'a [Period]) -> &'a Period
    {
        Self::date_to_period(periods, &self.work_order_dates.latest_allowed_finish_date)
    }

    // fn random_latest_periods(&mut self, periods: &[Period]) {
    //     let mut rng = thread_rng();
    //     let random_period = periods.choose(&mut rng).unwrap();
    //     self.work_order_dates.latest_allowed_finish_period =
    // random_period.clone(); }
}
impl FromStr for WorkOrderNumber
{
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let number = s.parse::<u64>()?;
        Ok(Self(number))
    }
}

pub type WorkOrderActivity = (WorkOrderNumber, ActivityNumber);

impl From<u64> for WorkOrderNumber
{
    fn from(value: u64) -> Self
    {
        WorkOrderNumber(value)
    }
}

impl Serialize for WorkOrderNumber
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkOrderNumber
{
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
impl WorkOrder
{
    pub fn work_order_test() -> Self
    {
        WorkOrder::builder(WorkOrderNumber(2100000001))
            .main_work_center(Resources::MtnMech)
            .operations_builder(10, Resources::Prodtech, |e| {
                e.operation_info(|oi| oi.number(1).work_remaining(10.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })
            .operations_builder(20, Resources::MtnMech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(20.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })
            .operations_builder(20, Resources::MtnMech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(30.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })
            .operations_builder(40, Resources::Prodtech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(40.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })
            .work_order_analytic_builder(|woab| {
                woab.system_status_codes(|sta| sta.rel(true))
                    .user_status_codes(|sta| sta.smat(true))
            })
            .work_order_info_builder(|woib| {
                // FIX [ ]
                // This is wrong and it should be fixed. You should make the
                // code work correctly no matter what.
                woib.priority(Priority::Int(1))
                    .work_order_type(WorkOrderType::Wdf(Priority::Int(1)))
            })
            .work_order_dates_builder(|wodb| {
                wodb.earliest_allowed_start_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
            })
            .build()
    }
}
#[cfg(test)]
mod tests
{
    use std::str::FromStr;

    use super::WorkOrder;
    // #[test]
    // fn test_initialize_work_load()
    // {
    //     let work_order = WorkOrder::work_order_test();

    //     assert_eq!(
    //         *work_order
    //             .work_order_load()
    //             .unwrap()
    //             .get(&Resources::from_str("PRODTECH").unwrap())
    //             .unwrap(),
    //         Work::from(50.0)
    //     );
    //     assert_eq!(
    //         *work_order
    //             .work_order_load()
    //             .unwrap()
    //             .get(&Resources::from_str("MTN-MECH").unwrap())
    //             .unwrap(),
    //         Work::from(50.0)
    //     );
    // }
    use super::*;

    #[test]
    fn test_date_to_period()
    {
        let periods: Vec<Period> = vec![
            Period::from_str("2024-W47-48").unwrap(),
            Period::from_str("2024-W49-50").unwrap(),
            Period::from_str("2024-W51-52").unwrap(),
            Period::from_str("2025-W1-2").unwrap(),
        ];

        let period_1 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2024, 12, 5).unwrap(),
        );
        let period_2 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2024, 12, 27).unwrap(),
        );
        let period_3 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
        );

        assert_eq!(period_1, periods.get(1).unwrap());
        assert_eq!(period_2, periods.get(2).unwrap());
        assert_eq!(period_3, periods.get(3).unwrap());
    }
}
