use serde::{Deserialize, Serialize};

use crate::Asset;

use super::StrategicObjectiveValueResponse;

use shared_types::agents::strategic::responses::StrategicObjectiveValueResponse;

use crate::agents::strategic_agent::StrategicObjectiveValue;

impl From<StrategicObjectiveValue> for StrategicObjectiveValueResponse {
    fn from(value: StrategicObjectiveValue) -> Self {
        todo!()
    }
}

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
