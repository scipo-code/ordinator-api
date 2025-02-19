pub mod algorithm;
pub mod display;
pub mod message_handlers;

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub struct StrategicOptions {
    number_of_removed_work_order: usize,
    rng: StdRng,
    urgency_weight: u64,
    resource_penalty_weight: u64,
    clustering_weight: u64,
}

impl Default for StrategicOptions {
    fn default() -> Self {
        StrategicOptions {
            number_of_removed_work_order: 50,
            rng: StdRng::from_os_rng(),
            urgency_weight: todo!(),
            resource_penalty_weight: todo!(),
            clustering_weight: todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct StrategicObjectiveValue {
    pub objective_value: u64,
    pub urgency: (u64, u64),
    pub resource_penalty: (u64, u64),
    pub clustering_value: (u64, u64),
}

impl StrategicObjectiveValue {
    pub fn new(strategic_options: &StrategicOptions) -> Self {
        Self {
            objective_value: 0,
            urgency: (strategic_options.urgency_weight, u64::MAX),
            resource_penalty: (strategic_options.resource_penalty_weight, u64::MAX),
            clustering_value: (strategic_options.clustering_weight, u64::MIN),
        }
    }

    pub fn aggregate_objectives(&mut self) {
        self.objective_value = self.urgency.0 * self.urgency.1
            + self.resource_penalty.0 * self.resource_penalty.1
            - self.clustering_value.0 * self.clustering_value.1;
    }
}

#[cfg(test)]
mod tests {

    use algorithm::strategic_parameters::StrategicClustering;
    use algorithm::ForcedWorkOrder;
    use anyhow::Result;
    use operation::OperationBuilder;
    use priority_queue::PriorityQueue;
    use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::scheduling_environment::worker_environment::resources::Id;
    use shared_types::scheduling_environment::SchedulingEnvironment;
    use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
    use shared_types::strategic::strategic_request_scheduling_message::StrategicRequestScheduling;
    use shared_types::strategic::OperationalResource;
    use shared_types::strategic::StrategicResources;
    use shared_types::Asset;
    use tests::algorithm::strategic_parameters::StrategicParameter;
    use tests::algorithm::strategic_parameters::StrategicParameters;
    use unloading_point::UnloadingPoint;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::Mutex;

    use crate::agents::traits::ActorBasedLargeNeighborhoodSearch;
    use crate::agents::traits::ObjectiveValueType;
    use crate::agents::Algorithm;
    use crate::agents::AlgorithmUtils;
    use crate::agents::ArcSwapSharedSolution;
    use crate::agents::StrategicSolution;

    use super::*;
    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use shared_types::scheduling_environment::work_order::operation::Operation;
    use shared_types::scheduling_environment::work_order::*;
    use shared_types::scheduling_environment::WorkOrders;

    use shared_types::scheduling_environment::time_environment::period::Period;

    #[test]
    fn test_extract_state_to_scheduler_overview() {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let unloading_point = UnloadingPoint::default();
        let operation_1 = OperationBuilder::new(
            ActivityNumber(10),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_2 = OperationBuilder::new(
            ActivityNumber(20),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_3 = OperationBuilder::new(
            ActivityNumber(30),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::work_order_test();

        let mut work_orders = WorkOrders::default();

        work_orders.insert(work_order_1);
    }

    #[test]
    fn test_update_scheduler_state() -> Result<()> {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicRequestScheduling::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::default()));

        let strategic_solution = StrategicSolution::new();

        let strategic_parameters =
            StrategicParameters::new(&Asset::Unknown, scheduling_environment.lock().unwrap())?;

        let mut strategic_algorithm = Algorithm::new(
            &Id::default(),
            strategic_solution,
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
        );

        let strategic_parameter = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            HashMap::new(),
        );

        strategic_algorithm
            .parameters
            .strategic_work_order_parameters
            .insert(work_order_number, strategic_parameter);

        strategic_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            strategic_algorithm
                .parameters
                .strategic_work_order_parameters
                .get(&work_order_number)
                .as_ref()
                .unwrap()
                .locked_in_period
                .as_ref()
                .unwrap()
                .period_string(),
            "2023-W47-48"
        );
        Ok(())
    }

    #[test]
    fn test_calculate_objective_value() -> Result<()> {
        let work_order_number = WorkOrderNumber(2100023841);

        let period = Period::from_str("2023-W49-50").unwrap();

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            vec![Resources::MtnMech, Resources::MtnElec, Resources::VenMech],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(40.0),
            vec![Resources::MtnScaf, Resources::MtnElec, Resources::VenMech],
        );
        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::default()));

        let mut strategic_parameters =
            StrategicParameters::new(&Asset::Unknown, scheduling_environment.lock().unwrap())?;

        let strategic_parameter = StrategicParameter::new(
            Some(period),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::from([(Resources::MtnMech, Work::from(10.0))]),
        );

        // QUESTION
        // Should you create a Dependency injection for the `SchedulingEnvironment`?
        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        let strategic_solution = StrategicSolution::new();
        let mut strategic_algorithm = Algorithm::new(
            &Id::default(),
            strategic_solution,
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
        );

        strategic_algorithm
            .solution
            .strategic_scheduled_work_orders
            .insert(work_order_number, None);

        strategic_algorithm
            .schedule_forced_work_order(&ForcedWorkOrder::Locked(work_order_number))?;

        let objective_value_type = strategic_algorithm.calculate_objective_value()?;

        let objective_value =
            if let ObjectiveValueType::Better(objective_value) = objective_value_type {
                objective_value
            } else {
                panic!();
            };

        strategic_algorithm.solution.objective_value = objective_value;

        assert_eq!(
            strategic_algorithm.solution.objective_value.objective_value, 2000,
            "{:#?}",
            strategic_algorithm.solution.objective_value
        );
        Ok(())
    }
}
