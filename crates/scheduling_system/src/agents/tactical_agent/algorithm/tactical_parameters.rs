use anyhow::Result;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::MutexGuard,
};

use chrono::NaiveDate;
use serde::Serialize;
use shared_types::{
    agents::tactical::TacticalResources,
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber, Operation, Work},
            ActivityRelation, WorkOrder, WorkOrderNumber,
        },
        worker_environment::{resources::Resources, EmptyFull},
        SchedulingEnvironment,
    },
};

use crate::agents::{tactical_agent::TacticalOptions, traits::Parameters};

#[derive(Default)]
pub struct TacticalParameters {
    pub tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter>,
    pub tactical_days: Vec<Day>,
    pub tactical_capacity: TacticalResources,
    pub options: TacticalOptions,
}

// TODO
// We should move all the code from the `AgentFactory` in here! That is the
// best option that we have.
impl Parameters for TacticalParameters {
    type Key = WorkOrderNumber;
    type Options = TacticalOptions;

    fn new(
        asset: &shared_types::Asset,
        options: Self::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let tactical_days = &scheduling_environment.time_environment.tactical_days;

        let tactical_capacity = scheduling_environment
            .worker_environment
            .generate_tactical_resources(tactical_days, EmptyFull::Full);

        let work_orders = scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(_, wo)| &wo.functional_location().asset == asset);

        let tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter> = work_orders
            .map(|(won, wo)| (*won, create_tactical_parameter(wo)))
            .collect();

        Ok(Self {
            tactical_work_orders,
            tactical_days: tactical_days.clone(),
            tactical_capacity,
            options,
        })
    }

    // We cannot reuse this component.
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    ) {
        todo!()
    }
}

// TODO
// We should think carefully about putting this into the `Parameters` trait as an
// associated function.
fn create_tactical_parameter(work_order: &WorkOrder) -> TacticalParameter {
    let operation_parameters = work_order
        .operations
        .iter()
        .map(|(acn, op)| {
            (
                *acn,
                OperationParameter::new(work_order.work_order_number, op),
            )
        })
        .collect::<HashMap<_, _>>();

    TacticalParameter::new(work_order, operation_parameters)
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
        work_order: &WorkOrder,
        operation_parameters: HashMap<ActivityNumber, OperationParameter>,
    ) -> Self {
        Self {
            main_work_center: work_order.main_work_center,
            tactical_operation_parameters: operation_parameters,
            weight: work_order.work_order_weight(),
            relations: work_order.relations.clone(),
            earliest_allowed_start_date: work_order.work_order_dates.earliest_allowed_start_date,
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
    pub fn new(work_order_number: WorkOrderNumber, operation: &Operation) -> Self {
        Self {
            work_order_number,
            number: operation.number(),
            // FIX
            // This should also have been created differently.
            duration: operation.duration().unwrap(),
            operating_time: operation.operating_time().unwrap(),
            work_remaining: operation.work_remaining().unwrap(),
            resource: *operation.resource(),
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
