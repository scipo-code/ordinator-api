use ordinator_actor_core::traits::ObjectiveValue;
use ordinator_scheduling_environment::time_environment::period::Period;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct StrategicObjectiveValueResponse
{
    field_one: String,
}

#[derive(Serialize)]
pub struct StrategicResponsePeriods
{
    periods: Vec<Period>,
}

impl StrategicResponsePeriods
{
    pub fn new(periods: Vec<Period>) -> Self
    {
        Self { periods }
    }
}

// This is a low level type and it should not be exposed here
// TODO [ ] FIX [ ]
// Make a custom type for the StrategicResourcesApi
#[derive(Serialize)]
pub enum StrategicResponseResources
{
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResourcesApi),
    Percentage(StrategicResourcesApi, StrategicResourcesApi),
}

#[derive(Serialize)]
struct StrategicResourcesApi {}

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseScheduling
{
    work_orders: usize,
    periods: Period,
}

impl StrategicResponseScheduling
{
    pub fn new(number_of_work_orders_changed: usize, period: Period) -> Self
    {
        Self {
            work_orders: number_of_work_orders_changed,
            periods: period,
        }
    }
}
use ordinator_scheduling_environment::Asset;

use crate::StrategicActor;

#[derive(Serialize)]
pub struct StrategicResponseStatus<T: ObjectiveValue>
{
    pub asset: Asset,
    pub strategic_objective_value: T,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl From<StrategicActor<Ss>> for StrategicResponseStatus
{
    fn from(value: StrategicActor<Ss>) -> Self
    {
        todo!()
    }
}

impl<T> StrategicResponseStatus<T>
where
    T: ObjectiveValue,
{
    pub fn new(
        asset: Asset,
        strategic_objective_value: T,
        number_of_strategic_work_orders: usize,
        number_of_periods: usize,
    ) -> Self
    {
        Self {
            asset,
            strategic_objective_value,
            number_of_strategic_work_orders,
            number_of_periods,
        }
    }
}
