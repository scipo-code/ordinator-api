use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use chrono::NaiveDate;
use serde::Serialize;
use shared_types::{
    scheduling_environment::{
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber, Work},
            ActivityRelation, WorkOrderNumber,
        },
        worker_environment::resources::Resources,
    },
    tactical::TacticalResources,
};

#[derive(Default, Clone)]
pub struct TacticalParameters {
    pub tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter>,
    pub tactical_capacity: TacticalResources,
}

#[derive(Clone, Serialize)]
pub struct TacticalParameter {
    pub main_work_center: Resources,
    pub tactical_operation_parameters: HashMap<ActivityNumber, OperationParameter>,
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: These two should be moved out of the pa
    pub earliest_allowed_start_date: NaiveDate,
}

impl TacticalParameter {
    pub fn new(
        main_work_center: Resources,
        operation_parameters: HashMap<ActivityNumber, OperationParameter>,
        weight: u64,
        relations: Vec<ActivityRelation>,
        earliest_allowed_start_date: NaiveDate,
    ) -> Self {
        Self {
            main_work_center,
            tactical_operation_parameters: operation_parameters,
            weight,
            relations,
            earliest_allowed_start_date,
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter {
    pub work_order_number: WorkOrderNumber,
    pub number: NumberOfPeople,
    pub duration: Work,
    pub operating_time: Work,
    pub work_remaining: Work,
    pub resource: Resources,
}

impl OperationParameter {
    pub fn new(
        work_order_number: WorkOrderNumber,
        number: NumberOfPeople,
        duration: Work,
        operating_time: Work,
        work_remaining: Work,
        resource: Resources,
    ) -> Self {
        Self {
            work_order_number,
            number,
            duration,
            operating_time,
            work_remaining,
            resource,
        }
    }
}

impl Display for OperationParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OperationParameters:\n
        {:?}\n
        number: {}\n
        duration: {}\n
        operating_time: {:?}\n
        work_remaining: {}\n
        resource: {}",
            self.work_order_number,
            self.number,
            self.duration,
            self.operating_time,
            self.work_remaining,
            self.resource
        )
    }
}
