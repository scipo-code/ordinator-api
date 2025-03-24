use std::collections::HashMap;
use std::collections::HashSet;

use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use serde::Deserialize;
use serde::Serialize;

use super::strategic_parameters::StrategicParameters;
use super::strategic_resources::OperationalResource;
use super::strategic_resources::StrategicResources;
use crate::StrategicOptions;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StrategicSolution
{
    pub objective_value: StrategicObjectiveValue,
    pub strategic_scheduled_work_orders: HashMap<WorkOrderNumber, Option<Period>>,
    pub strategic_loadings: StrategicResources,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct StrategicObjectiveValue
{
    pub objective_value: u64,
    pub urgency: (u64, u64),
    pub resource_penalty: (u64, u64),
    pub clustering_value: (u64, u64),
}

impl StrategicObjectiveValue
{
    pub fn new(strategic_options: &StrategicOptions) -> Self
    {
        Self {
            objective_value: 0,
            urgency: (strategic_options.urgency_weight, u64::MAX),
            resource_penalty: (strategic_options.resource_penalty_weight, u64::MAX),
            clustering_value: (strategic_options.clustering_weight, u64::MIN),
        }
    }

    pub fn aggregate_objectives(&mut self)
    {
        self.objective_value = self.urgency.0 * self.urgency.1
            + self.resource_penalty.0 * self.resource_penalty.1
            - self.clustering_value.0 * self.clustering_value.1;
    }
}
impl Solution for StrategicSolution
{
    type ObjectiveValue = StrategicObjectiveValue;
    type Parameters = StrategicParameters;

    fn new(parameters: &Self::Parameters) -> Self
    {
        let strategic_loadings = parameters
            .strategic_capacity
            .0
            .iter()
            .map(|(per, res)| {
                let inner_map: HashMap<_, _> = res
                    .iter()
                    .map(|(id, or)| {
                        (
                            id.clone(),
                            OperationalResource::new(
                                id,
                                Work::from(0.0),
                                or.skill_hours.keys().cloned().collect_vec(),
                            ),
                        )
                    })
                    .collect();

                (per.clone(), inner_map)
            })
            .collect::<HashMap<_, _>>();

        let strategic_loadings = StrategicResources::new(strategic_loadings);

        let strategic_scheduled_work_orders = parameters
            .strategic_work_order_parameters
            .keys()
            .map(|won| (*won, None))
            .collect();

        let strategic_objective_value = StrategicObjectiveValue::new(&parameters.strategic_options);
        Self {
            objective_value: strategic_objective_value,
            strategic_scheduled_work_orders,
            strategic_loadings,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue)
    {
        self.objective_value = other_objective_value;
    }
}

impl StrategicSolution
{
    pub fn supervisor_work_orders_from_strategic(
        &self,
        supervisor_periods: &[Period],
    ) -> HashSet<WorkOrderNumber>
    {
        let mut supervisor_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

        self.strategic_scheduled_work_orders
            .iter()
            .for_each(|(won, opt_per)| {
                if let Some(period) = opt_per {
                    if supervisor_periods.contains(period) {
                        supervisor_work_orders.insert(*won);
                    }
                }
            });
        supervisor_work_orders
    }
}
