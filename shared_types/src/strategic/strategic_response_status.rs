use crate::Asset;
use serde::{Deserialize, Serialize};

use super::StrategicObjectiveValue;

#[derive(Serialize, Deserialize)]
pub struct StrategicResponseStatus {
    pub asset: Asset,
    pub strategic_objective: StrategicObjectiveValue,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl StrategicResponseStatus {
    pub fn new(
        asset: Asset,
        strategic_objective: StrategicObjectiveValue,
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
