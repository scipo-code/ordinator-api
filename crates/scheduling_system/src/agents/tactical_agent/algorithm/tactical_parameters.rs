use anyhow::Result;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::MutexGuard,
};

use chrono::NaiveDate;
use serde::Serialize;
use shared_types::{
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber, Work},
            ActivityRelation, WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::Resources,
        SchedulingEnvironment,
    },
    tactical::TacticalResources,
};

use crate::agents::traits::Parameters;

#[derive(Default, Clone)]
pub struct TacticalParameters {
    pub tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter>,
    pub tactical_days: Vec<Day>,
    pub tactical_capacity: TacticalResources,
}

// TODO
// We should move all the code from the `AgentFactory` in here! That is the
// best option that we have.
impl Parameters for TacticalParameters {
    type Key = WorkOrderActivity;
    type Options = TacticalOptions;

    fn new(
        asset: &shared_types::Asset,
        options: Self::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let tactical_resources_from_file = scheduling_environment_guard
            .worker_environment
            .generate_tactical_resources(
                &scheduling_environment_guard.time_environment.tactical_days,
            );

        tactical_algorithm
            .tactical_parameters
            .tactical_capacity
            .update_resources(tactical_resources_from_file);

        tactical_algorithm.create_tactical_parameters(scheduling_environment_guard, asset);
        let tactical_resources_capacity =
            initialize_tactical_resources(scheduling_environment_guard, Work::from(0.0));

        Ok(Self {
            tactical_work_orders: todo!(),
            tactical_days: todo!(),
            tactical_capacity: todo!(),
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: std::sync::MutexGuard<
            shared_types::scheduling_environment::SchedulingEnvironment,
        >,
    ) {
        todo!()
    }
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
