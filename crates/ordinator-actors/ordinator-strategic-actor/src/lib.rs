mod algorithm;
pub mod messages;

use std::sync::RwLockReadGuard;

use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::time_environment::MaterialToPeriod;
use ordinator_scheduling_environment::work_order::WorkOrderConfigurations;
use rand::rngs::StdRng;

/// *question*
#[derive(Debug, PartialEq, Clone)]
pub struct StrategicOptions
{
    pub number_of_removed_work_order: usize,
    pub rng: StdRng,
    pub urgency_weight: u64,
    pub resource_penalty_weight: u64,
    pub clustering_weight: u64,
    pub work_order_configurations: WorkOrderConfigurations,
    pub material_to_period: MaterialToPeriod,
}
impl<'a> From<&RwLockReadGuard<'a, SystemConfigurations>> for StrategicOptions
{
    fn from(value: &RwLockReadGuard<SystemConfigurations>) -> Self
    {
        let number_of_removed_work_order = value
            .actor_configurations
            .strategic_options
            .number_of_removed_work_orders;
        let urgency_weight = value.actor_configurations.strategic_options.urgency_weight;
        let resource_penalty_weight = value
            .actor_configurations
            .strategic_options
            .resource_penalty_weight;
        let clustering_weight = value
            .actor_configurations
            .strategic_options
            .clustering_weight;
        let work_order_configurations = value.work_order_configurations;

        let material_to_period = value.material_to_period;

        let rng = StdRng::from_os_rng();
        // QUESTION [ ]
        // _Should this field be private or public?_
        //
        // You should provide an ID here to solve this problem.
        StrategicOptions {
            number_of_removed_work_order,
            rng,
            urgency_weight,
            resource_penalty_weight,
            clustering_weight,
            work_order_configurations,
            material_to_period,
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::Mutex;

    use algorithm::ForcedWorkOrder;
    use algorithm::strategic_resources::OperationalResource;
    use algorithm::strategic_resources::StrategicResources;
    use algorithm::strategic_solution::StrategicSolution;
    use anyhow::Result;
    use messages::requests::ScheduleChange;
    use messages::requests::StrategicRequestScheduling;
    use ordinator_actor_core::algorithm::Algorithm;
    use ordinator_scheduling_environment::SchedulingEnvironment;
    use ordinator_scheduling_environment::time_environment::period::Period;
    use ordinator_scheduling_environment::work_order::WorkOrder;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    use ordinator_scheduling_environment::work_order::WorkOrders;
    use ordinator_scheduling_environment::work_order::operation::Operation;
    use ordinator_scheduling_environment::work_order::operation::OperationBuilder;
    use ordinator_scheduling_environment::work_order::operation::Work;
    use ordinator_scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
    use ordinator_scheduling_environment::worker_environment::resources::Id;
    use ordinator_scheduling_environment::worker_environment::resources::Resources;
    use tests::algorithm::strategic_parameters::StrategicParameters;
    use tests::algorithm::strategic_parameters::WorkOrderParameter;

    use super::*;

    #[test]
    fn test_extract_state_to_scheduler_overview()
    {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let unloading_point = UnloadingPoint::default();
        let operation_1 = OperationBuilder::new(
            10,
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_2 = OperationBuilder::new(
            20,
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_3 = OperationBuilder::new(
            30,
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
    fn test_update_scheduler_state() -> Result<()>
    {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicRequestScheduling::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::default()));

        let id = Id::default();

        // WARN
        // This is an error. It should be implemented in a different way. I think
        // that. So the supervisors could be traits. You should think about how this
        // can be designed.
        let strategic_options = StrategicOptions::default();

        let strategic_parameters = StrategicParameters::new(
            &id,
            strategic_options,
            &scheduling_environment.lock().unwrap(),
        )?;

        let strategic_solution = StrategicSolution::new(&strategic_parameters);

        // let mut strategic_algorithm = Algorithm::new(
        //     &id,
        //     strategic_solution,
        //     strategic_parameters,
        //     ArcSwapSharedSolution::default().into(),
        // );

        // let strategic_parameter = WorkOrderParameter::new(
        //     Some(periods[0].clone()),
        //     HashSet::new(),
        //     periods.first().unwrap().clone(),
        //     1000,
        //     HashMap::new(),
        // );

        // strategic_algorithm
        //     .parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number, strategic_parameter);

        // strategic_algorithm
        //     .update_scheduling_state(strategic_scheduling_internal)
        //     .unwrap();

        // assert_eq!(
        //     strategic_algorithm
        //         .parameters
        //         .strategic_work_order_parameters
        //         .get(&work_order_number)
        //         .as_ref()
        //         .unwrap()
        //         .locked_in_period
        //         .as_ref()
        //         .unwrap()
        //         .period_string(),
        //     "2023-W47-48"
        // );
        Ok(())
    }

    #[test]
    fn test_calculate_objective_value() -> Result<()>
    {
        let work_order_number = WorkOrderNumber(2100023841);

        let period = Period::from_str("2023-W49-50").unwrap();

        let operational_resource_1 = OperationalResource::new("OP_TEST_0", Work::from(40.0), vec![
            Resources::MtnMech,
            Resources::MtnElec,
            Resources::VenMech,
        ]);
        let operational_resource_2 = OperationalResource::new("OP_TEST_1", Work::from(40.0), vec![
            Resources::MtnScaf,
            Resources::MtnElec,
            Resources::VenMech,
        ]);
        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::default()));

        let id = Id::default();

        let strategic_options = StrategicOptions::default();

        let mut strategic_parameters = StrategicParameters::new(
            &id,
            strategic_options,
            &scheduling_environment.lock().unwrap(),
        )?;

        let strategic_parameter = WorkOrderParameter::new(
            Some(period),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::from([(Resources::MtnMech, Work::from(10.0))]),
        );

        // QUESTION
        // Should you create a Dependency injection for the `SchedulingEnvironment`?
        // TODO
        // This should be created so that each type that implements `Parameters` have
        // an insert function.
        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        // let strategic_solution = StrategicSolution::new(&strategic_parameters);
        // let mut strategic_algorithm = Algorithm::new(
        //     &Id::default(),
        //     strategic_solution,
        //     strategic_parameters,
        //     ArcSwapSharedSolution::default().into(),
        // );

        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number, None);

        // strategic_algorithm
        //     .schedule_forced_work_order(&ForcedWorkOrder::Locked(work_order_number))?
        // ;

        // let objective_value_type = strategic_algorithm.calculate_objective_value()?;

        // let objective_value =
        //     if let ObjectiveValueType::Better(objective_value) = objective_value_type
        // {         objective_value
        //     } else {
        //         panic!();
        //     };

        // strategic_algorithm.solution.objective_value = objective_value;

        // assert_eq!(
        //     strategic_algorithm.solution.objective_value.objective_value, 2000,
        //     "{:#?}",
        //     strategic_algorithm.solution.objective_value
        // );
        Ok(())
    }
}
// use std::fmt::Display;

// TODO
// Make a generic display for `Agent` so that we can view all the different
// agent easily. impl Display for Agent {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "SchedulerAgent: \n
//             Platform: {}, \n",
//             self.asset,
//         )
//     }
// }
