use std::collections::HashMap;

use crate::models::time_environment::period::Period;
use crate::models::work_order::{
    order_type::WorkOrderType, priority::Priority, revision::Revision, status_codes::MaterialStatus,
};
use crate::Asset;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseStatus {
    pub asset: Asset,
    pub strategic_objective: f64,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl StrategicResponseStatus {
    pub fn new(
        asset: Asset,
        strategic_objective: f64,
        number_of_strategic_work_orders: usize,
        number_of_periods: usize,
    ) -> Self {
        Self {
            asset,
            strategic_objective,
            number_of_strategic_work_orders,
            number_of_periods,
        }
    }
}
#[derive(Serialize)]
pub struct WorkOrdersInPeriod {
    work_orders: HashMap<u32, WorkOrderResponse>,
}

impl WorkOrdersInPeriod {
    pub fn new(work_orders: HashMap<u32, WorkOrderResponse>) -> Self {
        Self { work_orders }
    }
}

#[derive(Serialize)]
pub struct WorkOrderResponse {
    earliest_period: Period,
    awsc: bool,
    sece: bool,
    revision: Revision,
    work_order_type: WorkOrderType,
    priority: Priority,
    vendor: bool,
    material: MaterialStatus,
}

impl WorkOrderResponse {
    pub fn new(
        earliest_period: Period,
        awsc: bool,
        sece: bool,
        revision: Revision,
        work_order_type: WorkOrderType,
        priority: Priority,
        vendor: bool,
        material: MaterialStatus,
    ) -> Self {
        Self {
            earliest_period,
            awsc,
            sece,
            revision,
            work_order_type,
            priority,
            vendor,
            material,
        }
    }
}
