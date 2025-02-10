pub mod algorithm;
pub mod display;
pub mod message_handlers;

use rand::rngs::StdRng;
use rand::SeedableRng;

pub struct StrategicOptions {
    number_of_removed_work_order: usize,
    rng: StdRng,
}

impl Default for StrategicOptions {
    fn default() -> Self {
        StrategicOptions {
            number_of_removed_work_order: 50,
            rng: StdRng::from_os_rng(),
        }
    }
}

#[cfg(test)]
mod tests {

    use algorithm::strategic_parameters::StrategicClustering;
    use algorithm::ForcedWorkOrder;
    use algorithm::StrategicAlgorithm;
    use anyhow::Result;
    use operation::OperationBuilder;
    use priority_queue::PriorityQueue;
    use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
    use shared_types::strategic::strategic_request_scheduling_message::StrategicRequestScheduling;
    use shared_types::strategic::OperationalResource;
    use shared_types::strategic::StrategicResources;
    use tests::algorithm::strategic_parameters::StrategicParameter;
    use tests::algorithm::strategic_parameters::StrategicParameters;
    use unloading_point::UnloadingPoint;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::agents::traits::ActorBasedLargeNeighborhoodSearch;
    use crate::agents::ArcSwapSharedSolution;

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
    fn test_update_scheduler_state() {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicRequestScheduling::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let optimized_work_orders = StrategicParameters::new(
            HashMap::new(),
            StrategicResources::default(),
            StrategicClustering::default(),
        );

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            PriorityQueue::new(),
            optimized_work_orders,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            HashMap::new(),
        );

        scheduler_agent_algorithm.set_strategic_parameter(work_order_number, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .strategic_parameters
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

        let mut strategic_parameters = StrategicParameters::new(
            HashMap::new(),
            strategic_resources,
            StrategicClustering::default(),
        );

        let strategic_parameter = StrategicParameter::new(
            Some(period),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::from([(Resources::MtnMech, Work::from(10.0))]),
        );

        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueue::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![],
        );

        strategic_algorithm
            .strategic_solution
            .strategic_scheduled_work_orders
            .insert(work_order_number, None);

        strategic_algorithm
            .schedule_forced_work_order(&ForcedWorkOrder::Locked(work_order_number))?;

        strategic_algorithm.calculate_objective_value()?;

        assert_eq!(
            strategic_algorithm
                .strategic_solution
                .objective_value
                .objective_value,
            2000
        );
        Ok(())
    }
}
