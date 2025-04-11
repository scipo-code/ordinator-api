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

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::builder().build()));

        let system_configuration = SystemConfigurations::read_all_configs().unwrap();
        let strategic_options = StrategicOptions::from((system_configuration, &Id::default()));

        let algorithm: StrategicAlgorithm<Ss> = Algorithm::builder()
            .id(Id::default())
            .parameters(strategic_options, &scheduling_environment.lock().unwrap());

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
