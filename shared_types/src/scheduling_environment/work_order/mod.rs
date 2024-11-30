pub mod display;
pub mod functional_location;
pub mod operation;
pub mod priority;
pub mod revision;
pub mod status_codes;
pub mod system_condition;
pub mod unloading_point;
pub mod work_order_dates;
pub mod work_order_text;
pub mod work_order_type;

use crate::scheduling_environment::work_order::functional_location::FunctionalLocation;
use crate::scheduling_environment::work_order::operation::Operation;
use crate::scheduling_environment::work_order::priority::Priority;
use crate::scheduling_environment::work_order::revision::Revision;
use crate::scheduling_environment::work_order::status_codes::MaterialStatus;
use crate::scheduling_environment::work_order::status_codes::SystemStatusCodes;
use crate::scheduling_environment::work_order::system_condition::SystemCondition;
use crate::scheduling_environment::work_order::unloading_point::UnloadingPoint;
use crate::scheduling_environment::work_order::work_order_dates::WorkOrderDates;
use crate::scheduling_environment::work_order::work_order_text::WorkOrderText;
use crate::scheduling_environment::work_order::work_order_type::WorkOrderType;
use chrono::{DateTime, Utc};
use operation::OperationBuilder;
use serde::{Deserialize, Serialize};
use status_codes::UserStatusCodes;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::scheduling_environment::worker_environment::resources::Resources;

use self::operation::ActivityNumber;
use self::operation::Work;

use super::time_environment::period::Period;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct WorkOrderNumber(pub u64);
impl WorkOrderNumber {
    pub fn is_dummy(&self) -> bool {
        self.0 == 0
    }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderInfo {
    pub priority: Priority,
    pub work_order_type: WorkOrderType,
    pub functional_location: FunctionalLocation,
    pub work_order_text: WorkOrderText,
    pub revision: Revision,
    pub system_condition: SystemCondition,
    pub work_order_info_detail: WorkOrderInfoDetail,
}

impl WorkOrderInfo {
    pub fn new(
        priority: Priority,
        work_order_type: WorkOrderType,
        functional_location: FunctionalLocation,
        work_order_text: WorkOrderText,
        revision: Revision,
        system_condition: SystemCondition,
        work_order_info_detail: WorkOrderInfoDetail,
    ) -> Self {
        WorkOrderInfo {
            priority,
            work_order_type,
            functional_location,
            work_order_text,
            revision,
            system_condition,
            work_order_info_detail,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    pub work_order_weight: u64,
    pub work_order_work: Work,
    pub work_load: HashMap<Resources, Work>,
    pub fixed: bool,
    pub vendor: bool,
    pub system_status_codes: SystemStatusCodes,
    pub user_status_codes: UserStatusCodes,
}

impl WorkOrderAnalytic {
    pub fn new(
        work_order_weight: u64,
        work_order_work: Work,
        work_load: HashMap<Resources, Work>,
        fixed: bool,
        vendor: bool,
        system_status_codes: SystemStatusCodes,
        user_status_codes: UserStatusCodes,
    ) -> Self {
        WorkOrderAnalytic {
            work_order_weight,
            work_order_work,
            work_load,
            fixed,
            vendor,
            system_status_codes,
            user_status_codes,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderInfoDetail {
    pub subnetwork: String,
    pub maintenance_plan: String,
    pub planner_group: String,
    pub maintenance_plant: String,
    pub pm_collective: String,
    pub room: String,
}

impl WorkOrderInfoDetail {
    pub fn new(
        subnetwork: String,
        maintenance_plan: String,
        planner_group: String,
        maintenance_plant: String,
        pm_collective: String,
        room: String,
    ) -> Self {
        Self {
            subnetwork,
            maintenance_plan,
            planner_group,
            maintenance_plant,
            pm_collective,
            room,
        }
    }
}

impl WorkOrder {
    pub fn new(
        work_order_number: WorkOrderNumber,
        main_work_center: Resources,
        operations: HashMap<ActivityNumber, Operation>,
        relations: Vec<ActivityRelation>,
        work_order_analytic: WorkOrderAnalytic,
        order_dates: WorkOrderDates,
        work_order_info: WorkOrderInfo,
    ) -> Self {
        WorkOrder {
            work_order_number,
            main_work_center,
            operations,
            relations,
            work_order_analytic,
            work_order_dates: order_dates,
            work_order_info,
        }
    }

    pub fn functional_location(&self) -> &FunctionalLocation {
        &self.work_order_info.functional_location
    }

    pub fn operations(&self) -> &HashMap<ActivityNumber, Operation> {
        &self.operations
    }

    pub fn work_order_number(&self) -> &WorkOrderNumber {
        &self.work_order_number
    }

    pub fn insert_operation(&mut self, operation: Operation) {
        self.operations.insert(operation.activity, operation);
    }

    pub fn order_dates_mut(&mut self) -> &mut WorkOrderDates {
        &mut self.work_order_dates
    }

    pub fn order_dates(&self) -> &WorkOrderDates {
        &self.work_order_dates
    }

    pub fn revision(&self) -> &Revision {
        &self.work_order_info.revision
    }

    pub fn work_order_type(&self) -> &WorkOrderType {
        &self.work_order_info.work_order_type
    }

    pub fn priority(&self) -> &Priority {
        &self.work_order_info.priority
    }

    pub fn work_load(&self) -> &HashMap<Resources, Work> {
        &self.work_order_analytic.work_load
    }

    pub fn work_order_weight(&self) -> u64 {
        self.work_order_analytic.work_order_weight
    }

    pub fn is_vendor(&self) -> bool {
        self.work_order_analytic.vendor
    }

    pub fn relations(&self) -> &Vec<ActivityRelation> {
        &self.relations
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

impl WorkOrder {
    pub fn initialize(&mut self, periods: &[Period]) {
        self.initialize_work_load();
        self.initialize_weight();
        self.initialize_vendor();
        self.initialize_material(periods);
        assert!(
            self.work_order_dates
                .earliest_allowed_start_period
                .end_date()
                .date_naive()
                >= self.work_order_dates.earliest_allowed_start_date
        );
        // TODO : Other fields
    }

    pub fn initialize_weight(&mut self) {
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

        // TODO Implement for VIS and ABC
    }

    pub fn initialize_work_load(&mut self) {
        let mut work_load: HashMap<Resources, Work> = HashMap::new();

        for (_, operation) in self.operations.iter() {
            *work_load
                .entry(operation.resource().clone())
                .or_insert(Work::from(0.0)) += operation.work_remaining().clone().unwrap();
        }

        self.work_order_analytic.work_order_work = work_load
            .clone()
            .into_values()
            .reduce(|acc, work| acc + work)
            .unwrap()
            .clone();
        self.work_order_analytic.work_load = work_load;
    }

    pub fn initialize_vendor(&mut self) {
        let work_load = self.work_load().clone();
        self.work_order_analytic.vendor = work_load
            .iter()
            .any(|(resource, _)| resource.is_ven_variant())
    }

    /// This method determines that earliest allow start date and period for the work order. This is
    /// a maximum of the material status and the earliest start period of the operations.
    /// TODO : A stance will have to be taken on the VEN, SHUTDOWN, and SUBNETWORKS.
    /// We will get an error here! The problem is that after this the EASD will not be contained
    /// anymore.
    fn initialize_material(&mut self, periods: &[Period]) {
        match &self.work_order_analytic.user_status_codes.clone().into() {
            MaterialStatus::Nmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[0]
                    .clone()
                    .max(self.work_order_dates.earliest_allowed_start_period.clone());
            }
            MaterialStatus::Smat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[0]
                    .clone()
                    .max(self.work_order_dates.earliest_allowed_start_period.clone());
            }
            MaterialStatus::Cmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[2]
                    .clone()
                    .max(self.work_order_dates.earliest_allowed_start_period.clone());
            }
            MaterialStatus::Pmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[3]
                    .clone()
                    .max(self.work_order_dates.earliest_allowed_start_period.clone());
            }
            MaterialStatus::Wmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[3]
                    .clone()
                    .max(self.work_order_dates.earliest_allowed_start_period.clone());
            }
            MaterialStatus::Unknown => {}
        }
    }
    pub fn find_excluded_periods(&self, periods: &[Period]) -> HashSet<Period> {
        let mut excluded_periods: HashSet<Period> = HashSet::new();
        for (i, period) in periods.iter().enumerate() {
            if *period < self.work_order_dates.earliest_allowed_start_period
                || (self.is_vendor() && i <= 3)
                || (self.revision().shutdown && i <= 3)
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
}

//TODO This should not be the default tr
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

        work_order.initialize_work_load();

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
