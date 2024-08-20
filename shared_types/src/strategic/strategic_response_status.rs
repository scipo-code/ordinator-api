use std::collections::{HashMap, HashSet};

use crate::scheduling_environment::time_environment::period::Period;
use crate::scheduling_environment::work_order::status_codes::StatusCodes;
use crate::scheduling_environment::work_order::{WorkOrderInfo, WorkOrderNumber};
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
pub struct WorkOrdersStatus {
    work_orders: HashMap<WorkOrderNumber, WorkOrderResponse>,
}

impl WorkOrdersStatus {
    pub fn new(work_orders: HashMap<WorkOrderNumber, WorkOrderResponse>) -> Self {
        Self { work_orders }
    }
}

#[derive(Serialize)]
pub struct WorkOrderResponse {
    earliest_period: Period,
    work_order_info: WorkOrderInfo,
    vendor: bool,
    weight: u64,
    status_codes: StatusCodes,
    optimized_work_order_response: Option<OptimizedWorkOrderResponse>,
}

impl WorkOrderResponse {
    pub fn new(
        earliest_period: Period,
        work_order_info: WorkOrderInfo,
        vendor: bool,
        weight: u64,
        status_codes: StatusCodes,
        optimized_work_order_response: Option<OptimizedWorkOrderResponse>,
    ) -> Self {
        Self {
            earliest_period,
            work_order_info,
            vendor,
            weight,
            status_codes,
            optimized_work_order_response,
        }
    }
}

#[derive(Serialize)]
pub struct OptimizedWorkOrderResponse {
    scheduled_period: Period,
    locked_in_period: Option<Period>,
    excluded_periods: HashSet<Period>,
    latest_period: Period,
}

impl OptimizedWorkOrderResponse {
    pub fn new(
        scheduled_period: Period,
        locked_in_period: Option<Period>,
        excluded_periods: HashSet<Period>,
        latest_period: Period,
    ) -> Self {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_periods,
            latest_period,
        }
    }
}
