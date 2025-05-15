use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::{self};
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use chrono::NaiveDate;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderConfigurations;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::TacticalOptions;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;

use super::tactical_resources::TacticalResources;

pub struct TacticalParameters
{
    pub tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter>,
    pub tactical_days: Vec<Day>,
    pub tactical_capacity: TacticalResources,
    pub tactical_options: TacticalOptions,
}

// TODO
// We should move all the code from the `AgentFactory` in here! That is the
// best option that we have.
impl Parameters for TacticalParameters
{
    type Key = WorkOrderNumber;

    fn from_source(
        id: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let tactical_options = &scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .with_context(|| {
                format!(
                    "Asset: {} is not present in the SchedulingEnvironment",
                    id.asset()
                )
            })?;

        let work_orders = scheduling_environment
            .work_orders
            .inner
            .iter()
            // WARN: Unwrap accepted. Every agent should always be connected to an Asset
            // QUESTION: Is this actually true?
            .filter(|(_, wo)| &wo.functional_location().asset == id.2.first().unwrap());

        // This is what you get from working with Rust, you get all the nice things and
        // then this is what you have to deal with. What is the correct approach
        // here? You need to understand what the goal of the code is to be able
        // to fix this.
        let tactical_capacity = TacticalResources::from((scheduling_environment, id));

        let tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter> = work_orders
            .map(|(won, wo)| {
                Ok((
                    *won,
                    // This should also be found inside of the database.
                    // There is something that has to be inverted here. You are not designing this
                    // is the best possible way.
                    create_tactical_parameter(wo, &tactical_options.work_order_configurations)?,
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, TacticalParameter>>>()?;

        let tactical_days = scheduling_environment.time_environment.days
            [0..tactical_options.tactical.number_of_tactical_days]
            .to_vec();
        Ok(Self {
            tactical_work_orders,
            tactical_days,
            tactical_capacity,
            tactical_options: tactical_options.tactical.tactical_options.clone(),
        })
    }

    // We cannot reuse this component.
    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

// TODO
// We should think carefully about putting this into the `Parameters` trait as
// an associated function. These `create_parameter` functions will be insanely
// important later on. Say every algorithm should have their own of these
// functions... No there should only be one and that one should accept a
// Generic?
//
// Is that even possible? I think that it is. Keep this up! You have to
// continue.
pub fn create_tactical_parameter(
    work_order: &WorkOrder,
    work_order_configuration: &WorkOrderConfigurations,
) -> Result<TacticalParameter>
{
    let operation_parameters = work_order
        .operations
        .0
        .iter()
        .map(|(acn, op)| {
            (
                *acn,
                OperationParameter::new(work_order.work_order_number, op),
            )
        })
        .collect::<HashMap<_, _>>();

    TacticalParameter::new(work_order, work_order_configuration, operation_parameters)
}

#[derive(Clone, Serialize)]
pub struct TacticalParameter
{
    pub main_work_center: Resources,
    pub tactical_operation_parameters: HashMap<ActivityNumber, OperationParameter>,
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: These two should be moved out of the pa
    pub earliest_allowed_start_date: NaiveDate,
}

// How should the parameters be build here?
impl TacticalParameter
{
    pub fn new(
        work_order: &WorkOrder,
        // This should be a part of the options.
        work_order_configuration: &WorkOrderConfigurations,
        operation_parameters: HashMap<ActivityNumber, OperationParameter>,
    ) -> Result<Self>
    {
        Ok(Self {
            main_work_center: work_order.main_work_center,
            tactical_operation_parameters: operation_parameters,
            weight: work_order.work_order_value(work_order_configuration)?,
            relations: work_order.operations.relations(),
            earliest_allowed_start_date: work_order.work_order_dates.earliest_allowed_start_date,
        })
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter
{
    pub work_order_number: WorkOrderNumber,
    pub number: NumberOfPeople,
    pub duration: Work,
    pub operating_time: Work,
    pub work_remaining: Work,
    pub resource: Resources,
}

impl OperationParameter
{
    pub fn new(work_order_number: WorkOrderNumber, operation: &Operation) -> Self
    {
        Self {
            work_order_number,
            number: operation.operation_info.number,
            // FIX
            // This should also have been created differently.
            duration: operation.operation_analytic.duration,
            operating_time: operation.operation_info.operating_time,
            work_remaining: operation.operation_info.work_remaining,
            resource: operation.resource,
        }
    }
}

impl Display for OperationParameter
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
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
