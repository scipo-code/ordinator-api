use std::collections::HashMap;
use std::fmt::Display;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::TacticalOptions;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

use super::tactical_parameters::TacticalParameters;
use super::tactical_resources::TacticalResources;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct TacticalObjectiveValue {
    pub objective_value: u64,
    pub urgency: (usize, u64),
    pub resource_penalty: (usize, u64),
}

impl TacticalObjectiveValue {
    pub fn new(tactical_options: &TacticalOptions) -> Self {
        Self {
            objective_value: 0,
            urgency: (tactical_options.urgency, u64::MAX),
            resource_penalty: (tactical_options.resource_penalty, u64::MAX),
        }
    }

    pub fn aggregate_objectives(&mut self) {
        self.objective_value = self.urgency.0 as u64 * self.urgency.1
            + self.resource_penalty.0 as u64 * self.resource_penalty.1;
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TacticalSolution {
    pub(crate) objective_value: TacticalObjectiveValue,
    pub(crate) tactical_work_orders: TacticalScheduledWorkOrders,
    pub(crate) tactical_loadings: TacticalResources,
}
// This should be put into the `algorithm.rs` file
impl Solution for TacticalSolution {
    type ObjectiveValue = TacticalObjectiveValue;
    type Parameters = TacticalParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        let tactical_loadings_inner: HashMap<Resources, Days> = parameters
            .tactical_capacity
            .resources
            .iter()
            .map(|(wo, days)| {
                let inner_map = days
                    .days
                    .keys()
                    .map(|day| (day.clone(), Work::from(0.0)))
                    .collect();
                (*wo, Days::new(inner_map))
            })
            .collect();

        let tactical_scheduled_work_orders_inner: HashMap<_, _> = parameters
            .tactical_work_orders
            .keys()
            .map(|won| (*won, WhereIsWorkOrder::NotScheduled))
            .collect();

        // You are still learning this.
        Self {
            objective_value: TacticalObjectiveValue::new(&parameters.tactical_options),
            tactical_work_orders: TacticalScheduledWorkOrders(tactical_scheduled_work_orders_inner),
            tactical_loadings: TacticalResources::new(tactical_loadings_inner),
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}

impl TacticalSolution {
    pub fn release_from_tactical_solution(&mut self, work_order_number: &WorkOrderNumber) {
        self.tactical_work_orders
            .0
            .insert(*work_order_number, WhereIsWorkOrder::Strategic);
    }

    pub fn tactical_scheduled_days(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> Result<&Vec<(Day, Work)>> {
        let tactical_day = &self
            .tactical_work_orders
            .0
            .get(work_order_number)
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not present in the tactical solution",
                    work_order_number
                )
            })?
            .tactical_operations()
            .with_context(|| {
                format!(
                    "WorkOrderNumber: {:?} was not scheduled for the tactical solution",
                    work_order_number
                )
            })?
            .0
            .get(activity_number)
            .with_context(|| {
                format!(
                    "ActivityNumber: {:?} was not present in the tactical solution",
                    activity_number
                )
            })?
            .scheduled;

        Ok(tactical_day)
    }

    pub fn tactical_insert_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        tactical_scheduled_operations: TacticalScheduledOperations,
    ) {
        self.tactical_work_orders.0.insert(
            work_order_number,
            WhereIsWorkOrder::Tactical(tactical_scheduled_operations),
        );
    }
}
// This is part of the solution. I think that you should rewrite the trait here
// so that you can work with the
#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalScheduledWorkOrders(
    pub HashMap<WorkOrderNumber, WhereIsWorkOrder<TacticalScheduledOperations>>,
);

// TODO [ ]
// Make a trait here to implement the type.
// This is basically an interface to the type that we need to implement this
// on. I think that the
pub trait TacticalWhereIsWorkOrder {
    fn is_tactical(&self) -> bool;

    fn tactical_operations(&self) -> Result<&TacticalScheduledOperations>;
}
impl TacticalWhereIsWorkOrder for WhereIsWorkOrder<TacticalScheduledOperations> {
    fn is_tactical(&self) -> bool {
        matches!(self, WhereIsWorkOrder::Tactical(_))
    }

    fn tactical_operations(&self) -> Result<&TacticalScheduledOperations> {
        match self {
            WhereIsWorkOrder::Strategic => bail!(
                "A call to extract the {} was made but received {}",
                std::any::type_name::<TacticalScheduledOperations>(),
                std::any::type_name_of_val(self),
            ),
            WhereIsWorkOrder::Tactical(tactical_scheduled_operations) => {
                Ok(tactical_scheduled_operations)
            }
            WhereIsWorkOrder::NotScheduled => bail!(
                "The work order has not been scheduled yet, you are most likely calling this method before complete initialization"
            ),
        }
    }
}

impl TacticalScheduledWorkOrders {
    pub fn scheduled_work_orders(&self) -> usize {
        self.0
            .iter()
            .filter(|(_won, sch_wo)| sch_wo.is_tactical())
            .count()
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TacticalScheduledOperations(pub HashMap<ActivityNumber, OperationSolution>);

//
impl TacticalScheduledOperations {
    pub fn insert_operation_solution(
        &mut self,
        activity: ActivityNumber,
        operation_solution: OperationSolution,
    ) {
        self.0.insert(activity, operation_solution);
    }
}

impl Display for TacticalScheduledOperations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tactical_operations = self.0.iter().collect::<Vec<_>>();
        tactical_operations
            .sort_by(|a, b| a.1.work_order_activity.1.cmp(&b.1.work_order_activity.1));

        for operation_solution in tactical_operations {
            write!(f, "activity: {:#?}", operation_solution.0)?;
            write!(f, "{}", operation_solution.1)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct TacticalSolutionBuilder(TacticalSolution);

#[allow(dead_code)]
impl TacticalSolutionBuilder {
    pub fn with_tactical_days(
        mut self,
        tactical_days: HashMap<WorkOrderNumber, WhereIsWorkOrder<TacticalScheduledOperations>>,
    ) -> Self {
        self.0.tactical_work_orders.0 = tactical_days;
        self
    }

    pub fn build(self) -> TacticalSolution {
        TacticalSolution {
            objective_value: self.0.objective_value,
            tactical_work_orders: self.0.tactical_work_orders,
            tactical_loadings: self.0.tactical_loadings,
        }
    }
}
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize)]
pub struct OperationSolution {
    pub scheduled: Vec<(Day, Work)>,
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_order_activity: WorkOrderActivity,
}

impl OperationSolution {
    pub fn new(
        scheduled: Vec<(Day, Work)>,
        resource: Resources,
        number: NumberOfPeople,
        work_remaining: Work,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> OperationSolution {
        OperationSolution {
            scheduled,
            resource,
            number,
            work_remaining,
            work_order_activity: (work_order_number, activity_number),
        }
    }
}

impl Display for OperationSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.work_order_activity)?;
        for scheduled in &self.scheduled {
            write!(f, "{} on {}", scheduled.1, scheduled.0)?
        }
        Ok(())
    }
}
