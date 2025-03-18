use crate::traits::Solution;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StrategicSolution {
    pub objective_value: StrategicObjectiveValue,
    pub strategic_scheduled_work_orders: HashMap<WorkOrderNumber, Option<Period>>,
    pub strategic_loadings: StrategicResources,
}

impl Solution for StrategicSolution {
    type ObjectiveValue = StrategicObjectiveValue;
    type Parameters = StrategicParameters;

    fn new(parameters: &Self::Parameters) -> Self {
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

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}

impl StrategicSolution {
    pub fn supervisor_work_orders_from_strategic(
        &self,
        supervisor_periods: &[Period],
    ) -> HashSet<WorkOrderNumber> {
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
