pub mod display;
pub mod operation;
pub mod work_order_analytic;
pub mod work_order_dates;
pub mod work_order_info;

use crate::scheduling_environment::work_order::operation::Operation;
use crate::scheduling_environment::work_order::work_order_analytic::status_codes::MaterialStatus;
use crate::scheduling_environment::work_order::work_order_analytic::status_codes::SystemStatusCodes;
use crate::scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
use crate::scheduling_environment::work_order::work_order_dates::WorkOrderDates;
use crate::scheduling_environment::work_order::work_order_info::functional_location::FunctionalLocation;
use crate::scheduling_environment::work_order::work_order_info::priority::Priority;
use crate::scheduling_environment::work_order::work_order_info::revision::Revision;
use crate::scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use crate::scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use crate::scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use chrono::{DateTime, Utc};
use colored::Colorize;
use operation::OperationBuilder;
use operation::OperationsBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::num::ParseIntError;
use std::str::FromStr;
use work_order_analytic::status_codes::UserStatusCodes;
use work_order_analytic::WorkOrderAnalytic;
use work_order_analytic::WorkOrderAnalyticBuilder;
use work_order_info::WorkOrderInfo;
use work_order_info::WorkOrderInfoDetail;

use crate::scheduling_environment::worker_environment::resources::Resources;

use self::operation::ActivityNumber;
use self::operation::Work;

use super::time_environment::period::Period;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder {
    pub work_order_number: WorkOrderNumber,
    pub main_work_center: Resources,
    pub operations: HashMap<ActivityNumber, Operation>,
    pub relations: Vec<ActivityRelation>,
    pub work_order_analytic: WorkOrderAnalytic,
    pub work_order_dates: WorkOrderDates,
    pub work_order_info: WorkOrderInfo,
}

pub struct WorkOrderBuilder {
    pub work_order_number: WorkOrderNumber,
    pub main_work_center: Resources,
    pub operations: Option<HashMap<ActivityNumber, Operation>>,
    // FIX
    // Every operation needs to have a relation between them. There
    // is no way around this. It should be an enforced invariant.
    pub relations: Vec<ActivityRelation>,
    pub work_order_analytic: WorkOrderAnalytic,
    pub work_order_dates: WorkOrderDates,
    pub work_order_info: WorkOrderInfo,
}

impl WorkOrderBuilder {
    pub fn build(self) -> WorkOrder {
        WorkOrder {
            work_order_number: self.work_order_number,
            main_work_center: self.main_work_center,
            operations: self.operations.unwrap_or_default(),
            relations: self.relations,
            work_order_analytic: self.work_order_analytic,
            work_order_dates: self.work_order_dates,
            work_order_info: self.work_order_info,
        }
    }

    pub fn work_order_number(&mut self, work_order_number: WorkOrderNumber) -> &mut Self {
        self.work_order_number = work_order_number;
        self
    }

    pub fn main_work_center(&mut self, main_work_center: Resources) -> &mut Self {
        self.main_work_center = main_work_center;
        self
    }

    pub fn operations_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut OperationsBuilder) -> &mut OperationsBuilder,
    {
        let mut operations_builder = OperationsBuilder::new();

        f(&mut operations_builder);

        self.operations = Some(operations_builder.build());
        self
    }

    pub fn work_order_analytic_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderAnalyticBuilder) -> &mut WorkOrderAnalyticBuilder,
    {
        let work_order_analytic_builder = WorkOrderAnalyticBuilder::new();

        f(&mut work_order_analytic_builder);

        self.work_order_analytic = work_order_analytic_builder.build();
        self
    }

    pub fn work_order_dates<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderDatesBuilder) -> &mut WorkOrderDatesBuilder,
    {
        let work_order_analytic_builder = WorkOrderDatesBuilder::new();

        f(&mut work_order_analytic_builder);

        self.work_order_dates = work_order_analytic_builder.build();
        self
    }

    pub fn work_order_info<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrderInfoBuilder) -> &mut WorkOrderInfoBuilder,
    {
        let work_order_info_builder = WorkOrderInfoBuilder::new();

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
#[derive(serde::Deserialize, Debug)]
pub struct WeightParams {
    order_type_weights: HashMap<String, u64>,
    status_weights: HashMap<String, u64>,
    vis_priority_map: HashMap<char, u64>,
    wdf_priority_map: HashMap<u64, u64>,
    wgn_priority_map: HashMap<u64, u64>,
    wpm_priority_map: HashMap<char, u64>,
}

impl WeightParams {
    pub fn read_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("WORK_ORDER_WEIGHTINGS").expect("Work Order configuration parameters should always be provided through configuraion files specified in the .env file");
        let config_contents = fs::read_to_string(config_path).expect("Could not read config file");

        let config: WeightParams = serde_json::from_str(&config_contents)?;

        Ok(config)
    }
}

// You can remove all the initialization logic! So cool! I am not sure about the
// builder though.
impl WorkOrder {
    pub fn initialize(&mut self, periods: &[Period]) {
        self.work_order_weight();
        // FIX
        // self.random_latest_periods(periods);
        // FIX
        assert!(
            self.work_order_dates
                .earliest_allowed_start_period
                .end_date()
                .date_naive()
                >= self.work_order_dates.earliest_allowed_start_date
        );
        // TODO : Other fields
    }

    pub fn work_order_weight(&mut self) {
        // FIX
        // This should be removed. Where should the global configs be read
        // from? I am not really sure. You have done a lot today! I think that
        // reading more. Is a really good idea. Maybe finish the one you started
        // quickly.
        // TODO [ ]
        // There can be no stray `configs` like these! They have to be handled
        // in a higher level.
        let parameters: WeightParams = WeightParams::read_config().unwrap();
        self.work_order_analytic.work_order_weight = 0;

        match &self.work_order_info.work_order_type {
            WorkOrderType::Wdf(wdf_priority) => match wdf_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    self.work_order_analytic.work_order_weight +=
                        parameters.wdf_priority_map[int] * parameters.order_type_weights["WDF"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                Priority::Int(int) if (&0..&8).contains(&int) => {
                    self.work_order_analytic.work_order_weight +=
                        parameters.wgn_priority_map[int] * parameters.order_type_weights["WGN"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    self.work_order_analytic.work_order_weight +=
                        parameters.wpm_priority_map[char] * parameters.order_type_weights["WPM"]
                }
                _ => panic!("Received a wrong input number work order priority"),
            },
            WorkOrderType::Wro(_) => (),
            WorkOrderType::Other => {
                self.work_order_analytic.work_order_weight += parameters.order_type_weights["Other"]
            }
        };

        if self.work_order_analytic.user_status_codes.awsc {
            self.work_order_analytic.work_order_weight += parameters.status_weights["AWSC"];
        }

        if self.work_order_analytic.user_status_codes.sece {
            self.work_order_analytic.work_order_weight += parameters.status_weights["SECE"];
        }

        if self.work_order_analytic.system_status_codes.pcnf
            && self.work_order_analytic.system_status_codes.nmat
            || self.work_order_analytic.user_status_codes.smat
        {
            self.work_order_analytic.work_order_weight +=
                parameters.status_weights["PCNF_NMAT_SMAT"];
        }

        self.work_order_analytic.work_order_weight *=
            self.work_order_analytic.work_order_work.to_f64() as u64

        // TODO [ ]
        // Shame yourself for writing so horrible code! It is actually disgusting, you
        // must rewrite all the horrible parts.
        // Implement for VIS and ABC
        //
        //
    }

    pub fn work_order_load(&mut self) -> HashMap<Resources, Work> {
        self.operations
            .values()
            .fold(HashMap::default(), |mut acc, ele_opr| {
                *acc.entry(*ele_opr.resource()).or_insert(Work::from(0.0)) +=
                    ele_opr.work_remaining().unwrap();
                acc
            })
    }

    pub fn vendor(&mut self) -> bool {
        for operation in self.operations.values() {
            if operation.resource.is_ven_variant() {
                return true;
            }
        }
        false
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
            if *period < self.work_order_dates.earliest_allowed_start_period
                || (self.work_order_analytic.vendor && i <= 3)
                || (self.work_order_info.revision.shutdown && i <= 3)
            {
                assert!(
                    self.work_order_dates
                        .earliest_allowed_start_period
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
        self.operations.insert(operation.activity, operation);
    }

    pub fn work_order_weight(&self) -> u64 {
        self.work_order_analytic.work_order_weight
    }

    pub fn earliest_allowed_start_period<'a>(&'a mut self, periods: &'a [Period]) -> &'a Period {
        // This whole thing is bull shit.
        match &self.work_order_analytic.user_status_codes.clone().into() {
            MaterialStatus::Nmat => {
                (&periods[0]).max(&self.work_order_dates.earliest_allowed_start_period)
            }
            MaterialStatus::Smat => {
                (&periods[0]).max(&self.work_order_dates.earliest_allowed_start_period)
            }
            MaterialStatus::Cmat => {
                (&periods[2]).max(&self.work_order_dates.earliest_allowed_start_period)
            }
            MaterialStatus::Pmat => {
                (&periods[3]).max(&self.work_order_dates.earliest_allowed_start_period)
            }
            MaterialStatus::Wmat => {
                (&periods[3]).max(&self.work_order_dates.earliest_allowed_start_period)
            }
            MaterialStatus::Unknown => panic!("WorkOrder does not have a material status"),
        }
    }

    pub fn unloading_point_contains_period(&self, clone: Period) -> bool {
        for operation in &self.operations {
            if operation.1.unloading_point.period == Some(clone.clone()) {
                return true;
            }
        }
        false
    }

    pub fn unloading_point(&self) -> Option<Period> {
        self.operations
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
        let mut operations = HashMap::new();

        let unloading_point = UnloadingPoint::default();
        let operation_0010 = OperationBuilder::new(
            ActivityNumber(10),
            unloading_point.clone(),
            Resources::Prodtech,
            Some(Work::from(10.0)),
        )
        .build();
        let operation_0020 = OperationBuilder::new(
            ActivityNumber(20),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(20.0)),
        )
        .build();
        let operation_0030 = OperationBuilder::new(
            ActivityNumber(30),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(30.0)),
        )
        .build();
        let operation_0040 = OperationBuilder::new(
            ActivityNumber(40),
            unloading_point.clone(),
            Resources::Prodtech,
            Some(Work::from(40.0)),
        )
        .build();

        operations.insert(ActivityNumber(10), operation_0010);
        operations.insert(ActivityNumber(20), operation_0020);
        operations.insert(ActivityNumber(30), operation_0030);
        operations.insert(ActivityNumber(40), operation_0040);

        let work_order_analytic = WorkOrderAnalytic::new(
            1000,
            Work::from(100.0),
            HashMap::new(),
            false,
            false,
            SystemStatusCodes::default(),
            UserStatusCodes::default(),
        );

        let work_order_info = WorkOrderInfo::new(
            Priority::new_int(1),
            WorkOrderType::Wdf(Priority::dyn_new(Box::new(1_u64))),
            FunctionalLocation::default(),
            WorkOrderText::default(),
            Revision::default(),
            SystemCondition::Unknown,
            WorkOrderInfoDetail::default(),
        );

        WorkOrder::new(
            WorkOrderNumber(2100000001),
            Resources::MtnMech,
            operations,
            Vec::new(),
            work_order_analytic,
            WorkOrderDates::new_test(),
            work_order_info,
        )
    }
}
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use crate::scheduling_environment::worker_environment::resources::Resources;

    use super::{
        functional_location::FunctionalLocation,
        operation::{ActivityNumber, OperationBuilder, Work},
        priority::Priority,
        revision::Revision,
        status_codes::{SystemStatusCodes, UserStatusCodes},
        system_condition::SystemCondition,
        unloading_point::UnloadingPoint,
        work_order_dates::WorkOrderDates,
        work_order_text::WorkOrderText,
        work_order_type::WorkOrderType,
        WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderInfoDetail, WorkOrderNumber,
    };

    #[test]
    fn test_initialize_work_load() {
        let mut work_order = WorkOrder::new_test();

        work_order.work_order_load();

        dbg!(work_order.work_load());
        assert_eq!(
            *work_order
                .work_load()
                .get(&Resources::from_str("PRODTECH").unwrap())
                .unwrap(),
            Work::from(50.0)
        );
        assert_eq!(
            *work_order
                .work_load()
                .get(&Resources::from_str("MTN-MECH").unwrap())
                .unwrap(),
            Work::from(50.0)
        );
    }

    impl WorkOrder {
        pub fn new_test() -> Self {
            let mut operations = HashMap::new();

            let unloading_point = UnloadingPoint::default();
            let operation_0010 = OperationBuilder::new(
                ActivityNumber(10),
                unloading_point.clone(),
                Resources::Prodtech,
                Some(Work::from(10.0)),
            )
            .build();
            let operation_0020 = OperationBuilder::new(
                ActivityNumber(20),
                unloading_point.clone(),
                Resources::MtnMech,
                Some(Work::from(20.0)),
            )
            .build();
            let operation_0030 = OperationBuilder::new(
                ActivityNumber(30),
                unloading_point.clone(),
                Resources::MtnMech,
                Some(Work::from(30.0)),
            )
            .build();
            let operation_0040 = OperationBuilder::new(
                ActivityNumber(40),
                unloading_point.clone(),
                Resources::Prodtech,
                Some(Work::from(40.0)),
            )
            .build();

            operations.insert(ActivityNumber(10), operation_0010);
            operations.insert(ActivityNumber(20), operation_0020);
            operations.insert(ActivityNumber(30), operation_0030);
            operations.insert(ActivityNumber(40), operation_0040);

            let work_order_analytic = WorkOrderAnalytic::new(
                1000,
                Work::from(100.0),
                HashMap::new(),
                false,
                false,
                SystemStatusCodes::default(),
                UserStatusCodes::default(),
            );

            let work_order_info = WorkOrderInfo::new(
                Priority::new_int(1),
                WorkOrderType::Wdf(Priority::dyn_new(Box::new(1_u64))),
                FunctionalLocation::default(),
                WorkOrderText::default(),
                Revision::default(),
                SystemCondition::Unknown,
                WorkOrderInfoDetail::default(),
            );

            WorkOrder::new(
                WorkOrderNumber(2100023841),
                Resources::MtnMech,
                operations,
                Vec::new(),
                work_order_analytic,
                WorkOrderDates::new_test(),
                work_order_info,
            )
        }
    }
}
