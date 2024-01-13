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
use crate::models::work_order::order_dates::OrderDates;
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

use super::worker_environment::resources::Resources;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder {
    pub order_number: u32,
    // pub optimized_work_order: OptimizedWorkOrder,
    pub fixed: bool,
    pub order_weight: u32,
    pub priority: Priority,
    pub order_work: f64,
    pub operations: HashMap<u32, Operation>,
    pub work_load: HashMap<Resources, f64>,
    pub start_start: Vec<bool>,
    pub finish_start: Vec<bool>,
    pub postpone: Vec<DateTime<Utc>>,
    pub order_type: WorkOrderType,
    pub system_condition: SystemCondition,
    pub status_codes: StatusCodes,
    pub order_dates: OrderDates,
    pub revision: Revision,
    pub unloading_point: UnloadingPoint,
    pub functional_location: FunctionalLocation,
    pub order_text: OrderText,
    pub vendor: bool,
}

impl WorkOrder {
    #[cfg(test)]
    pub fn new(
        order_number: u32,
        fixed: bool,
        order_weight: u32,
        priority: Priority,
        order_work: f64,
        operations: HashMap<u32, Operation>,
        work_load: HashMap<Resources, f64>,
        start_start: Vec<bool>,
        finish_start: Vec<bool>,
        postpone: Vec<DateTime<Utc>>,
        order_type: WorkOrderType,
        system_condition: SystemCondition,
        status_codes: StatusCodes,
        order_dates: OrderDates,
        revision: Revision,
        unloading_point: UnloadingPoint,
        functional_location: FunctionalLocation,
        order_text: OrderText,
        vendor: bool,
    ) -> Self {
        WorkOrder {
            order_number,
            fixed,
            order_weight,
            priority,
            order_work,
            operations,
            work_load,
            start_start,
            finish_start,
            postpone,
            order_type,
            system_condition,
            status_codes,
            order_dates,
            revision,
            unloading_point,
            functional_location,
            order_text,
            vendor,
        }
    }

    pub fn get_work_order_number(&self) -> u32 {
        self.order_number
    }

    pub fn insert_operation(&mut self, operation: Operation) {
        self.operations.insert(operation.activity, operation);
    }
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
        let default_path = "parameters/work_order_weight_parameters.json";
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| default_path.to_string());
        let config_contents = fs::read_to_string(config_path).expect("Could not read config file");

        let config: WeightParam = serde_json::from_str(&config_contents)?;

        Ok(config)
    }
}

impl WorkOrder {
    pub fn initialize(&mut self) {
        self.initialize_weight();
        self.initialize_work_load();
        // TODO : Other fields
    }

    pub fn initialize_weight(&mut self) {
        let parameters: WeightParam = WeightParam::read_config().unwrap();

        self.order_weight = 0;

        match &self.order_type {
            WorkOrderType::Wdf(wdf_priority) => match wdf_priority {
                WDFPriority::One => {
                    self.order_weight +=
                        parameters.wdf_priority_map["1"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Two => {
                    self.order_weight +=
                        parameters.wdf_priority_map["2"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Three => {
                    self.order_weight +=
                        parameters.wdf_priority_map["3"] * parameters.order_type_weights["WDF"]
                }
                WDFPriority::Four => {
                    self.order_weight +=
                        parameters.wdf_priority_map["4"] * parameters.order_type_weights["WDF"]
                }
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                WGNPriority::One => {
                    self.order_weight +=
                        parameters.wgn_priority_map["1"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Two => {
                    self.order_weight +=
                        parameters.wgn_priority_map["2"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Three => {
                    self.order_weight +=
                        parameters.wgn_priority_map["3"] * parameters.order_type_weights["WGN"]
                }
                WGNPriority::Four => {
                    self.order_weight +=
                        parameters.wgn_priority_map["4"] * parameters.order_type_weights["WGN"]
                }
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                WPMPriority::A => {
                    self.order_weight +=
                        parameters.wpm_priority_map["A"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::B => {
                    self.order_weight +=
                        parameters.wpm_priority_map["B"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::C => {
                    self.order_weight +=
                        parameters.wpm_priority_map["C"] * parameters.order_type_weights["WPM"]
                }
                WPMPriority::D => {
                    self.order_weight +=
                        parameters.wpm_priority_map["D"] * parameters.order_type_weights["WPM"]
                }
            },
            WorkOrderType::Other => self.order_weight += parameters.order_type_weights["Other"],
        }

        if self.status_codes.awsc {
            self.order_weight += parameters.status_weights["AWSC"];
        }

        if self.status_codes.sece {
            self.order_weight += parameters.status_weights["SECE"];
        }

        if self.status_codes.pcnf && self.status_codes.material_status == MaterialStatus::Nmat
            || self.status_codes.material_status == MaterialStatus::Smat
        {
            self.order_weight += parameters.status_weights["PCNF_NMAT_SMAT"];
        }

        // TODO Implement for VIS and ABC
    }

    pub fn initialize_work_load(&mut self) {

        let mut work_load: HashMap<Resources, f64> = HashMap::new();

        for (_, operation) in self.operations.iter() {
            *work_load
                .entry(operation.work_center.clone())
                .or_insert(0.0) += operation.work_remaining;
        }

        self.work_load = work_load;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::worker_environment::resources::Resources;

    use super::{
        functional_location::FunctionalLocation,
        operation::Operation,
        order_dates::OrderDates,
        order_text::OrderText,
        order_type::{WDFPriority, WorkOrderType},
        priority::Priority,
        revision::Revision,
        status_codes::StatusCodes,
        system_condition::SystemCondition,
        unloading_point::UnloadingPoint,
        WorkOrder,
    };

    #[test]
    fn test_initialize_work_load() {
        let mut work_order = WorkOrder::new_test();

        work_order.initialize_work_load();

        assert_eq!(
            *work_order
                .work_load
                .get(&Resources::new_from_string("PRODTECH".to_string()))
                .unwrap(),
            50.0
        );
        assert_eq!(
            *work_order
                .work_load
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

            WorkOrder::new(
                2100023841,
                false,
                1000,
                Priority::new_int(1),
                100.0,
                operations,
                HashMap::new(),
                vec![],
                vec![],
                vec![],
                WorkOrderType::Wdf(WDFPriority::new(1)),
                crate::models::work_order::system_condition::SystemCondition::Unknown,
                StatusCodes::new_default(),
                OrderDates::new_test(),
                Revision::new_default(),
                UnloadingPoint::new_default(),
                FunctionalLocation::new_default(),
                OrderText::new_default(),
                false,
            )
        }
    }

    impl Default for WorkOrder {
        fn default() -> Self {
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

            WorkOrder::new(
                2100000001,
                false,
                1000,
                Priority::new_int(1),
                100.0,
                operations,
                HashMap::new(),
                vec![],
                vec![],
                vec![],
                WorkOrderType::Wdf(WDFPriority::new(1)),
                SystemCondition::Unknown,
                StatusCodes::new_default(),
                OrderDates::new_test(),
                Revision::new_default(),
                UnloadingPoint::new_default(),
                FunctionalLocation::new_default(),
                OrderText::new_default(),
                false,
            )
        }
    }
}
