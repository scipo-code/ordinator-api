pub mod operation;
pub mod order_dates;
pub mod order_text;
pub mod status_codes;
pub mod functional_location;
pub mod unloading_point;
pub mod revision;
pub mod order_type;
pub mod display;
pub mod priority;
pub mod optimized_work_order;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::models::work_order::operation::Operation;
use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::revision::Revision;
use crate::models::work_order::order_type::WorkOrderType;
use crate::models::work_order::priority::Priority;
use crate::models::work_order::optimized_work_order::OptimizedWorkOrder;
use crate::models::work_order::{order_type::{WDFPriority, WGNPriority, WPMPriority}, status_codes::MaterialStatus};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct WorkOrder {
    pub order_number: u32,
    pub optimized_work_order: OptimizedWorkOrder,
    pub fixed: bool,
    pub order_weight: u32,
    pub priority: Priority,
    pub order_work: f64,
    pub operations: HashMap<u32, Operation>,
    pub work_load: HashMap<String, f64>, 
    pub start_start: Vec<bool>,
    pub finish_start: Vec<bool>,
    pub postpone: Vec<DateTime<Utc>>,
    pub order_type: WorkOrderType,
    pub status_codes: StatusCodes,  
    pub order_dates: OrderDates,
    pub revision: Revision,
    pub unloading_point: UnloadingPoint, 
    pub functional_location: FunctionalLocation, 
    pub order_text: OrderText,
    pub vendor: bool,
}

impl WorkOrder {
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
        let config_contents = fs::read_to_string("parameters/work_order_weight_parameters.json").expect("Could not read config file");

        let config: WeightParam = serde_json::from_str(&config_contents)?;

        Ok(config)
    }
}

impl WorkOrder {

    pub fn initialize(&mut self) {
        dbg!("Initializing Work Orders");
        self.initialize_weight();
        self.initialize_work_load();
        // TODO : Other fields
    }

    pub fn initialize_weight(&mut self) {
        dbg!("Initializing Work Orders");

        let parameters: WeightParam = WeightParam::read_config().unwrap();

        self.order_weight = 0;

        match &self.order_type {
            WorkOrderType::WDF(wdf_priority) => match wdf_priority {
                WDFPriority::One => self.order_weight += parameters.wdf_priority_map["1"] * parameters.order_type_weights["WDF"],
                WDFPriority::Two => self.order_weight += parameters.wdf_priority_map["2"] * parameters.order_type_weights["WDF"],
                WDFPriority::Three => self.order_weight += parameters.wdf_priority_map["3"] * parameters.order_type_weights["WDF"],
                WDFPriority::Four => self.order_weight += parameters.wdf_priority_map["4"] * parameters.order_type_weights["WDF"],
            },
            WorkOrderType::WGN(wgn_priority) => match wgn_priority {
                WGNPriority::One => self.order_weight += parameters.wgn_priority_map["1"] * parameters.order_type_weights["WGN"],
                WGNPriority::Two => self.order_weight += parameters.wgn_priority_map["2"] * parameters.order_type_weights["WGN"],
                WGNPriority::Three => self.order_weight += parameters.wgn_priority_map["3"] * parameters.order_type_weights["WGN"],
                WGNPriority::Four => self.order_weight += parameters.wgn_priority_map["4"] * parameters.order_type_weights["WGN"]
            },	                
            WorkOrderType::WPM(wpm_priority) => match wpm_priority {
                WPMPriority::A => self.order_weight += parameters.wpm_priority_map["A"] * parameters.order_type_weights["WPM"],
                WPMPriority::B => self.order_weight += parameters.wpm_priority_map["B"] * parameters.order_type_weights["WPM"],
                WPMPriority::C => self.order_weight += parameters.wpm_priority_map["C"] * parameters.order_type_weights["WPM"],
                WPMPriority::D => self.order_weight += parameters.wpm_priority_map["D"] * parameters.order_type_weights["WPM"]
            },
            WorkOrderType::Other => self.order_weight += parameters.order_type_weights["Other"],
        }  

        if self.status_codes.awsc {
            self.order_weight += parameters.status_weights["AWSC"];
        }

        if self.status_codes.sece {
            self.order_weight += parameters.status_weights["SECE"];
        }

        if self.status_codes.pcnf && self.status_codes.material_status == MaterialStatus::Nmat || self.status_codes.material_status == MaterialStatus::Smat {
            self.order_weight += parameters.status_weights["PCNF_NMAT_SMAT"];
        }

        // TODO Implement for VIS and ABC

    }  

    pub fn initialize_work_load(&mut self) {
        dbg!("Initializing Work Orders");

        let mut work_load: HashMap<String, f64> = HashMap::new();

        for (_, operation) in self.operations.iter() {
            work_load.insert(operation.work_center.clone(), operation.work_remaining);
        }

        self.work_load = work_load;
    }
}
