use serde::{Deserialize, Serialize};

pub mod functional_location;
pub mod priority;
pub mod revision;
pub mod system_condition;
pub mod work_order_text;
pub mod work_order_type;

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
