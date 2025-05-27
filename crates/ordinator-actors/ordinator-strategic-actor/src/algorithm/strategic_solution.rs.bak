use std::collections::HashMap;

use anyhow::Result;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::StrategicOptions;
use serde::Deserialize;
use serde::Serialize;

use super::strategic_parameters::StrategicParameters;
use super::strategic_resources::OperationalResource;
use super::strategic_resources::StrategicResources;

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
    pub urgency: (usize, u64),
    pub resource_penalty: (usize, u64),
    pub clustering_value: (usize, u64),
}

impl StrategicObjectiveValue

{
    pub fn new(strategic_options: &StrategicOptions) -> Self
    {
        Self {
            objective_value: u64::MAX,
            urgency: (strategic_options.urgency_weight, u64::MIN),
            resource_penalty: (strategic_options.resource_penalty_weight, u64::MIN),
            clustering_value: (strategic_options.clustering_weight, u64::MIN),
        }
    }

    pub fn aggregate_objectives(&mut self)
    {
        self.objective_value = self.urgency.0 as u64 * self.urgency.1
            + self.resource_penalty.0 as u64 * self.resource_penalty.1
            - self.clustering_value.0 as u64 * self.clustering_value.1;
    }
}
impl Solution for StrategicSolution
{
    type ObjectiveValue = StrategicObjectiveValue;
    type Parameters = StrategicParameters;

    fn new(parameters: &Self::Parameters) -> Result<Self>
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
                                or.skill_hours.keys().cloned().collect(),
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

        // Motherfucker. Should the parameters have the options or not? This is a
        // crucial question. I think that they should I am not sure what I
        // should do here. This code is horrible... You have to do better, you
        // need more faith... You have to remain calm in this.
        // QUESTION
        // Should the options be inside of the parameters or used as a dependency
        // injected variable? I think that the best approach here is to make the
        // code function. The issue is that this becomes very complex, You need to
        // do it in a consistent way across all the different actors.
        //
        //
        let strategic_objective_value = StrategicObjectiveValue::new(&parameters.strategic_options);
        Ok(Self {
            objective_value: strategic_objective_value,
            strategic_scheduled_work_orders,
            strategic_loadings,
        })
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue)
    {
        self.objective_value = other_objective_value;
    }
}

impl<Ss> SwapSolution<Ss> for StrategicSolution
where
    Ss: SystemSolutions<Strategic = StrategicSolution>,
{
    fn swap(
        id: &ordinator_scheduling_environment::worker_environment::resources::Id,
        solution: Self,
        system_solution: &mut Ss,
    )
    {
        system_solution.strategic_swap(id, solution);
    }
}
