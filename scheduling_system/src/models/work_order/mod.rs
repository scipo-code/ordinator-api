pub mod display;
pub mod functional_location;
pub mod operation;
pub mod order_dates;
pub mod order_text;
pub mod order_type;
pub mod priority;
pub mod revision;
pub mod status_codes;
pub mod system_condition;
pub mod unloading_point;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::operation::Operation;
use crate::models::work_order::order_dates::WorkOrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::order_type::WorkOrderType;
use crate::models::work_order::priority::Priority;
use crate::models::work_order::revision::Revision;
use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::system_condition::SystemCondition;
use crate::models::work_order::unloading_point::UnloadingPoint;
// use crate::models::work_order::optimized_work_order::OptimizedWorkOrder;
use crate::models::work_order::{
    order_type::{WDFPriority, WGNPriority, WPMPriority},
    status_codes::MaterialStatus,
};

use shared_messages::resources::Resources;

use super::time_environment::period::Period;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder {
    work_order_number: u32,
    operations: HashMap<u32, Operation>,
    relations: Vec<ActivityRelation>,
    work_order_analytic: WorkOrderAnalytic,
    order_dates: WorkOrderDates,
    work_order_info: WorkOrderInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderInfo {
    priority: Priority,
    work_order_type: WorkOrderType,
    functional_location: FunctionalLocation,
    order_text: OrderText,
    unloading_point: UnloadingPoint,
    revision: Revision,
    system_condition: SystemCondition,
}

impl WorkOrderInfo {
    pub fn new(
        priority: Priority,
        work_order_type: WorkOrderType,
        functional_location: FunctionalLocation,
        order_text: OrderText,
        unloading_point: UnloadingPoint,
        revision: Revision,
        system_condition: SystemCondition,
    ) -> Self {
        WorkOrderInfo {
            priority,
            work_order_type,
            functional_location,
            order_text,
            unloading_point,
            revision,
            system_condition,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    order_weight: u32,
    work_order_work: f64,
    work_load: HashMap<Resources, f64>,
    fixed: bool,
    vendor: bool,
    status_codes: StatusCodes,
}

impl WorkOrderAnalytic {
    pub fn new(
        order_weight: u32,
        work_order_work: f64,
        work_load: HashMap<Resources, f64>,
        fixed: bool,
        vendor: bool,
        status_codes: StatusCodes,
    ) -> Self {
        WorkOrderAnalytic {
            order_weight,
            work_order_work,
            work_load,
            fixed,
            vendor,
            status_codes,
        }
    }
}

impl WorkOrder {
    pub fn new(
        work_order_number: u32,
        operations: HashMap<u32, Operation>,
        relations: Vec<ActivityRelation>,
        work_order_analytic: WorkOrderAnalytic,
        order_dates: WorkOrderDates,
        work_order_info: WorkOrderInfo,
    ) -> Self {
        WorkOrder {
            work_order_number,
            operations,
            relations,
            work_order_analytic,

            order_dates,
            work_order_info,
        }
    }

    pub fn operations(&self) -> &HashMap<u32, Operation> {
        &self.operations
    }

    pub fn work_order_number(&self) -> &u32 {
        &self.work_order_number
    }

    pub fn insert_operation(&mut self, operation: Operation) {
        self.operations.insert(operation.activity, operation);
    }

    pub fn unloading_point(&self) -> &UnloadingPoint {
        &self.work_order_info.unloading_point
    }

    pub fn order_dates_mut(&mut self) -> &mut WorkOrderDates {
        &mut self.order_dates
    }

    pub fn order_dates(&self) -> &WorkOrderDates {
        &self.order_dates
    }

    pub fn status_codes(&self) -> &StatusCodes {
        &self.work_order_analytic.status_codes
    }

    pub fn revision(&self) -> &Revision {
        &self.work_order_info.revision
    }

    pub fn order_type(&self) -> &WorkOrderType {
        &self.work_order_info.work_order_type
    }

    pub fn priority(&self) -> &Priority {
        &self.work_order_info.priority
    }

    pub fn work_load(&self) -> &HashMap<Resources, f64> {
        &self.work_order_analytic.work_load
    }

    pub fn work_order_weight(&self) -> u32 {
        self.work_order_analytic.order_weight
    }

    pub fn is_vendor(&self) -> bool {
        self.work_order_analytic.vendor
    }

    pub fn relations(&self) -> &Vec<ActivityRelation> {
        &self.relations
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityRelation {
    StartStart,
    FinishStart,
    Postpone(DateTime<Utc>),
}

#[derive(Serialize, Deserialize)]
struct WeightParam {
    wdf_priority_map: std::collections::HashMap<String, u32>,
    wgn_priority_map: std::collections::HashMap<String, u32>,
    wpm_priority_map: std::collections::HashMap<String, u32>,
    vis_priority_map: std::collections::HashMap<String, u32>,
    order_type_weights: std::collections::HashMap<String, u32>,
    status_weights: std::collections::HashMap<String, u32>,
}

impl WeightParam {
    fn read_config() -> Result<Self, Box<dyn std::error::Error>> {
        let default_path = "scheduling_system/parameters/work_order_weight_parameters.json";
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| default_path.to_string());
        let config_contents = fs::read_to_string(config_path).expect("Could not read config file");

        let config: WeightParam = serde_json::from_str(&config_contents)?;

        Ok(config)
    }
}

impl WorkOrder {
    pub fn initialize(&mut self, periods: &[Period]) {
        self.initialize_work_load();
        self.initialize_weight();
        self.initialize_vendor();
        self.initialize_material(periods);
        // TODO : Other fields
    }

    pub fn initialize_weight(&mut self) {
        let parameters: WeightParam = WeightParam::read_config().unwrap();
        self.work_order_analytic.order_weight = 0;

        match &self.work_order_info.work_order_type {
            WorkOrderType::Wdf(wdf_priority) => match wdf_priority {
                WDFPriority::One => {
                    self.work_order_analytic.order_weight +=
                        parameters.wdf_priority_map["1"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Two => {
                    self.work_order_analytic.order_weight +=
                        parameters.wdf_priority_map["2"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Three => {
                    self.work_order_analytic.order_weight +=
                        parameters.wdf_priority_map["3"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Four => {
                    self.work_order_analytic.order_weight +=
                        parameters.wdf_priority_map["4"] * parameters.order_type_weights["WDF"]
                }
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                WGNPriority::One => {
                    self.work_order_analytic.order_weight +=
                        parameters.wgn_priority_map["1"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Two => {
                    self.work_order_analytic.order_weight +=
                        parameters.wgn_priority_map["2"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Three => {
                    self.work_order_analytic.order_weight +=
                        parameters.wgn_priority_map["3"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Four => {
                    self.work_order_analytic.order_weight +=
                        parameters.wgn_priority_map["4"] * parameters.order_type_weights["WGN"]
                }
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                WPMPriority::A => {
                    self.work_order_analytic.order_weight +=
                        parameters.wpm_priority_map["A"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::B => {
                    self.work_order_analytic.order_weight +=
                        parameters.wpm_priority_map["B"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::C => {
                    self.work_order_analytic.order_weight +=
                        parameters.wpm_priority_map["C"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::D => {
                    self.work_order_analytic.order_weight +=
                        parameters.wpm_priority_map["D"] * parameters.order_type_weights["WPM"]
                }
            },
            WorkOrderType::Wro(_) => (),
            WorkOrderType::Other => {
                self.work_order_analytic.order_weight += parameters.order_type_weights["Other"]
            }
        };

        if self.work_order_analytic.status_codes.awsc {
            self.work_order_analytic.order_weight += parameters.status_weights["AWSC"];
        }

        if self.work_order_analytic.status_codes.sece {
            self.work_order_analytic.order_weight += parameters.status_weights["SECE"];
        }

        if self.work_order_analytic.status_codes.pcnf
            && self.work_order_analytic.status_codes.material_status == MaterialStatus::Nmat
            || self.work_order_analytic.status_codes.material_status == MaterialStatus::Smat
        {
            self.work_order_analytic.order_weight += parameters.status_weights["PCNF_NMAT_SMAT"];
        }
        self.work_order_analytic.order_weight *=
            self.work_order_analytic.work_order_work.round() as u32;

        // TODO Implement for VIS and ABC
    }

    pub fn initialize_work_load(&mut self) {
        let mut work_load: HashMap<Resources, f64> = HashMap::new();

        for (_, operation) in self.operations.iter() {
            *work_load.entry(operation.resource.clone()).or_insert(0.0) += operation.work_remaining;
        }

        self.work_order_analytic.work_order_work = work_load.values().sum();
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
    fn initialize_material(&mut self, periods: &[Period]) {
        match self.status_codes().material_status {
            MaterialStatus::Nmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[0].clone();
            }
            MaterialStatus::Smat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[0].clone();
            }
            MaterialStatus::Cmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[2].clone();
            }
            MaterialStatus::Pmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[3].clone();
            }
            MaterialStatus::Wmat => {
                self.order_dates_mut().earliest_allowed_start_period = periods[3].clone();
            }
            MaterialStatus::Unknown => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use shared_messages::resources::Resources;

    use super::{
        functional_location::FunctionalLocation,
        operation::Operation,
        order_dates::WorkOrderDates,
        order_text::OrderText,
        order_type::{WDFPriority, WorkOrderType},
        priority::Priority,
        revision::Revision,
        status_codes::StatusCodes,
        system_condition::SystemCondition,
        unloading_point::UnloadingPoint,
        WorkOrder, WorkOrderAnalytic, WorkOrderInfo,
    };

    #[test]
    fn test_initialize_work_load() {
        let mut work_order = WorkOrder::new_test();

        work_order.initialize_work_load();

        assert_eq!(
            *work_order
                .work_load()
                .get(&Resources::new_from_string("PRODTECH".to_string()))
                .unwrap(),
            50.0
        );
        assert_eq!(
            *work_order
                .work_load()
                .get(&Resources::new_from_string("MTN-MECH".to_string()))
                .unwrap(),
            50.0
        );
    }

    impl WorkOrder {
        pub fn new_test() -> Self {
            let mut operations = HashMap::new();

            let operation_0010 =
                Operation::new_test(10, Resources::new_from_string("PRODTECH".to_string()), 10.0);
            let operation_0020 =
                Operation::new_test(20, Resources::new_from_string("MTN-MECH".to_string()), 20.0);
            let operation_0030 =
                Operation::new_test(30, Resources::new_from_string("MTN-MECH".to_string()), 30.0);
            let operation_0040 =
                Operation::new_test(40, Resources::new_from_string("PRODTECH".to_string()), 40.0);

            operations.insert(10, operation_0010);
            operations.insert(20, operation_0020);
            operations.insert(30, operation_0030);
            operations.insert(40, operation_0040);

            let work_order_analytic = WorkOrderAnalytic::new(
                1000,
                100.0,
                HashMap::new(),
                false,
                false,
                StatusCodes::new_default(),
            );

            let work_order_info = WorkOrderInfo::new(
                Priority::new_int(1),
                WorkOrderType::Wdf(WDFPriority::new(1)),
                FunctionalLocation::new_default(),
                OrderText::new_default(),
                UnloadingPoint::new_default(),
                Revision::new_default(),
                SystemCondition::Unknown,
            );

            WorkOrder::new(
                2100023841,
                operations,
                Vec::new(),
                work_order_analytic,
                WorkOrderDates::new_test(),
                work_order_info,
            )
        }
    }

    impl Default for WorkOrder {
        fn default() -> Self {
            let mut operations = HashMap::new();

            let operation_0010 = Operation::new_test(10, Resources::Prodtech, 10.0);
            let operation_0020 = Operation::new_test(20, Resources::MtnMech, 20.0);
            let operation_0030 = Operation::new_test(30, Resources::MtnMech, 30.0);
            let operation_0040 = Operation::new_test(40, Resources::Prodtech, 40.0);

            operations.insert(10, operation_0010);
            operations.insert(20, operation_0020);
            operations.insert(30, operation_0030);
            operations.insert(40, operation_0040);

            let work_order_analytic = WorkOrderAnalytic::new(
                1000,
                100.0,
                HashMap::new(),
                false,
                false,
                StatusCodes::new_default(),
            );

            let work_order_info = WorkOrderInfo::new(
                Priority::new_int(1),
                WorkOrderType::Wdf(WDFPriority::new(1)),
                FunctionalLocation::new_default(),
                OrderText::new_default(),
                UnloadingPoint::new_default(),
                Revision::new_default(),
                SystemCondition::Unknown,
            );

            WorkOrder::new(
                2100000001,
                operations,
                Vec::new(),
                work_order_analytic,
                WorkOrderDates::new_test(),
                work_order_info,
            )
        }
    }
}
