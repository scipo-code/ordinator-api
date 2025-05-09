use ordinator_actor_core::traits::ObjectiveValue;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::time_environment::period::Period;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct StrategicObjectiveValueResponse {
    field_one: String,
}

#[derive(Serialize)]
pub struct StrategicResponsePeriods {
    periods: Vec<Period>,
}

impl StrategicResponsePeriods {
    pub fn new(periods: Vec<Period>) -> Self {
        Self { periods }
    }
}

// This is a low level type and it should not be exposed here
// TODO [ ] FIX [ ]
// Make a custom type for the StrategicResourcesApi
#[derive(Serialize)]
pub enum StrategicResponseResources {
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResourcesApi),
    Percentage(StrategicResourcesApi, StrategicResourcesApi),
}

#[derive(Serialize)]
struct StrategicResourcesApi {}

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseScheduling {
    work_orders: usize,
    periods: Period,
}

impl StrategicResponseScheduling {
    pub fn new(number_of_work_orders_changed: usize, period: Period) -> Self {
        Self {
            work_orders: number_of_work_orders_changed,
            periods: period,
        }
    }
}
use ordinator_scheduling_environment::Asset;

use crate::StrategicActor;
use crate::algorithm::strategic_solution::StrategicSolution;

#[derive(Serialize)]
pub struct StrategicResponseStatus {
    pub asset: Asset,
    pub strategic_objective_value: usize,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl<Ss> From<&mut StrategicActor<Ss>> for StrategicResponseStatus
where
    Ss: SystemSolutionTrait<Strategic = StrategicSolution>,
{
    fn from(value: &mut StrategicActor<Ss>) -> Self {
        let strategic_parameters = &value.algorithm.parameters;

        let number_of_strategic_work_orders =
            strategic_parameters.strategic_work_order_parameters.len();

        let asset = value.actor_id.asset();

        let number_of_periods = value.algorithm.parameters.strategic_periods.len();

        // You need to generate a trait for the `objective value`
        // I would rather want to have a `from` implementation on this type than
        // make something like this.
        StrategicResponseStatus {
            number_of_strategic_work_orders,
            number_of_periods,
            asset: asset.clone(),
            strategic_objective_value: value.algorithm.solution.objective_value.objective_value
                as usize,
        }
    }
}
