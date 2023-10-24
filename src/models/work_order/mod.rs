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

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
<<<<<<< HEAD
use std::fs;
=======
use std::fmt;
>>>>>>> origin

use crate::models::work_order::operation::Operation;
use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::status_codes::StatusCodes;
use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::revision::Revision;
use crate::models::work_order::order_type::WorkOrderType;
use crate::models::work_order::priority::Priority;

<<<<<<< HEAD
use self::{order_type::{WDFPriority, WGNPriority, WPMPriority}, status_codes::MaterialStatus};

#[derive(Clone)]
=======
#[derive(Serialize, Deserialize)]
pub enum Priority {
    IntValue(i32),
    StringValue(String),
}

>>>>>>> origin
#[derive(Serialize, Deserialize)]
pub struct WorkOrder {
    pub order_number: u32,
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

<<<<<<< HEAD
#[derive(Serialize, Deserialize)]
struct WeightParam {
    WDF_priority_map: std::collections::HashMap<String, u32>,
    WGN_priority_map: std::collections::HashMap<String, u32>,
    WPM_priority_map: std::collections::HashMap<String, u32>,
    VIS_priority_map: std::collections::HashMap<String, u32>,
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
    pub fn calculate_weight(&mut self) {
        let parameters: WeightParam = WeightParam::read_config().unwrap();

        self.order_weight = 0;

        match &self.order_type {
            WorkOrderType::WDF(WDFPriority) => match WDFPriority {
                WDFPriority::One => self.order_weight += parameters.WDF_priority_map["1"] * parameters.order_type_weights["WDF"],
                WDFPriority::Two => self.order_weight += parameters.WDF_priority_map["2"] * parameters.order_type_weights["WDF"],
                WDFPriority::Three => self.order_weight += parameters.WDF_priority_map["3"] * parameters.order_type_weights["WDF"],
                WDFPriority::Four => self.order_weight += parameters.WDF_priority_map["4"] * parameters.order_type_weights["WDF"],
            },
            WorkOrderType::WGN(WGNPriority) => match WGNPriority {
                WGNPriority::One => self.order_weight += parameters.WGN_priority_map["1"] * parameters.order_type_weights["WGN"],
                WGNPriority::Two => self.order_weight += parameters.WGN_priority_map["2"] * parameters.order_type_weights["WGN"],
                WGNPriority::Three => self.order_weight += parameters.WGN_priority_map["3"] * parameters.order_type_weights["WGN"],
                WGNPriority::Four => self.order_weight += parameters.WGN_priority_map["4"] * parameters.order_type_weights["WGN"]
            },	                
            WorkOrderType::WPM(WPMPriority) => match WPMPriority {
                WPMPriority::A => self.order_weight +=parameters.WPM_priority_map["A"] * parameters.order_type_weights["WPM"],
                WPMPriority::B => self.order_weight +=parameters.WPM_priority_map["B"] * parameters.order_type_weights["WPM"],
                WPMPriority::C => self.order_weight +=parameters.WPM_priority_map["C"] * parameters.order_type_weights["WPM"],
                WPMPriority::D => self.order_weight +=parameters.WPM_priority_map["D"] * parameters.order_type_weights["WPM"]
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
}

// function create_weights(orders::OrderData)
//    priority_map = Dict(
//          "A"   =>  10000, 
//          "B"   =>    100, 
//          "C"   =>     10, 
//          "D"   =>      1, 
//          "1"   =>  10000, 
//          "2"   =>    100, 
//          "3"   =>     10, 
//          "4"   =>      1,
//          "V"   =>    100,
//          "I"   =>     10,
//          "S"   =>     10)

//    weight = zeros(Int, orders.operations)
//    for k in 1:orders.operations
//       if orders.order_type[k] == "WDF"
//          weight[k] += 10 * priority_map[orders.priority[k]]
//       end

//       if orders.order_type[k] == "WGN"
//          weight[k] += 8 * priority_map[orders.priority[k]]
//       end

//       if orders.order_type[k] == "WPM"
//          weight[k] += 5 * priority_map[orders.priority[k]]
//       end

//       if occursin(r"(SECE)", orders.order_user_status[k])
//          weight[k] += 75000
//       end
     
//       if occursin(r"(PCNF)", orders.order_system_status[k]) && occursin(r"(NMAT)", orders.order_user_status[k]) && occursin(r"(SMAT)", orders.order_user_status[k])
//          weight[k] += 15000
//       end

//       if occursin(r"(AWSC)", orders.order_user_status[k])
//          weight[k] += 100000
//       end
//       # weight[k] += (ismissing(orders.abc[k]) ? 0 : 10 * priority_map[orders.abc[k]])
//    end
//    return weight
// end
=======
impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order Number: {}, \nNumber of activities: {}, \nVendor: {}, \nAWSC: {}, \nShutdown", self.order_number, self.operations.len(), self.vendor, self.status_codes.AWSC)
    }
}
>>>>>>> origin
