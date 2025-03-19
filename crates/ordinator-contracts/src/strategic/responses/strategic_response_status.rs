use ordinator_scheduling_environment::Asset;
use serde::Deserialize;
use serde::Serialize;

use super::StrategicObjectiveValueResponse;

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseStatus {
    pub asset: Asset,
    pub strategic_objective_value: StrategicObjectiveValueResponse,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl StrategicResponseStatus {
    pub fn new(
        asset: Asset,
        strategic_objective_value: StrategicObjectiveValueResponse,
        number_of_strategic_work_orders: usize,
        number_of_periods: usize,
    ) -> Self {
        Self {
            asset,
            strategic_objective_value,
            number_of_strategic_work_orders,
            number_of_periods,
        }
    }
}
