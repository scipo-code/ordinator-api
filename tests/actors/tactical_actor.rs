#[test]
fn test_calculate_objective_value()
{
    let work_order_number = WorkOrderNumber(2100000001);
    let activity_number = 1;
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let tactical_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };
    // Work Order
    // Resources::MtnMech,
    // 10,
    // vec![],
    // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),

    // Operation
    // 1,
    // Work::from(1.0),
    // Work::from(1.0),
    // Work::from(1.0),
    // Resources::MtnMech,
    // The same goes for the `SchedulingEnvironment`. It should not be possible to
    // simply work on it without an arc mutex. Yes I think that is the best
    // appraoch.
    let scheduling_environment = SchedulingEnvironment::builder()
        .time_environment_builder(|ib| ib.tactical_days("2025-02-22T07:00:00Z", 56))
        .build();

    // TODO
    // Insert the needed functions here to create the `SchedulingEnvironment`

    let id = Id::default();

    let system_configurations = SystemConfigurations::read_all_configs().unwrap().load();

    // You need to make the this test in the integration testing, as you need a
    // correct way of initializing the `SharedSolution`
    let algorithm: TacticalAlgorithm = Algorithm::builder()
        .id(id)
        .parameters_and_solution(
            &system_configurations,
            &scheduling_environment.lock().unwrap(),
        )
        .unwrap()
        .build();

    // TODO [ ]
    // Which Options should be inserted into this? I think that the best
    // You should make a method on the
    // SystemConfigurations::strategic_options(...)
    // -> StrategicOptions TODO [ ]
    // Put the system configuration into the Orchestrator
    // TODO [ ]
    // Put the system configuration into the Agents
    // TODO [ ]
    // Make methods on the `SystemConfiguration` to extract the required
    // configurations.

    // let mut tactical_algorithm = Algorithm::new(
    //     &id,
    //     solution,
    //     parameters,
    //     ArcSwapSharedSolution::default().into(),
    // );

    // // This whole thing is ugly. Remember, you should work on getting the
    // configs // into the program, not the other way around.

    // // FIX
    // // This does not confine to the correct interface setup of the
    // program. You // should think about this in the code. What
    // other thing could you do // here?
    // let operation_parameter = OperationParameter::new(work_order_number,
    // operation);

    // let operation_solution = OperationSolution::new(
    //     vec![(
    //         tactical_algorithm.tactical_days[27].clone(),
    //         Work::from(1.0),
    //     )],
    //     Resources::MtnMech,
    //     operation_parameter.number,
    //     operation_parameter.work_remaining,
    //     work_order_number,
    //     activity_number,
    // );

    // let mut operation_parameters = HashMap::new();
    // operation_parameters.insert(activity_number, operation_parameter);

    // let mut operation_solutions = HashMap::new();
    // operation_solutions.insert(1, operation_solution);

    // // We simply have to make
    // let optimized_tactical_work_order =
    //     TacticalParameter::new(&work_order, operation_parameters);

    // tactical_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, optimized_tactical_work_order);

    // tactical_algorithm.calculate_objective_value().unwrap();

    // // assert_eq!(tactical_algorithm.objective_value().0, 270);
}

// This is ugly... I think that the best think to do here
#[test]
fn test_schedule_1()
{
    let work_order_number = WorkOrderNumber(2100000001);
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let tactical_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };

    // Should you work on test? Or getting the system operational? I think
    // that getting it operational is the best choice here. I do not
    // see a different way of doing it.
    // You should also make these test at somepoint.
    // QUESTION
    // You should make the test later comment them out. The issue with
    // starting to creating them now is that you will have to make
    // some thing of a You will have to comment them out, and then
    // introduce them back in again. let mut tactical_algorithm =
    // Algorithm::builder().new(     tactical_days(56),
    //     TacticalResources::new_from_data(
    //         Resources::iter().collect(),
    //         tactical_days(56),
    //         Work::from(0.0),
    //     ),
    //     TacticalResources::new_from_data(
    //         Resources::iter().collect(),
    //         tactical_days(56),
    //         Work::from(0.0),
    //     ),
    //     ArcSwapSharedSolution::default().into(),
    // );
    // // Work Order
    // // Resources::MtnMech,
    // // 10,
    // // vec![],
    // // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),

    // // Operation
    // // 1,
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Resources::MtnMech,

    // let operation_parameter = OperationParameter::new(work_order_number,
    // operation);

    // let mut tactical_operation_parameters = HashMap::new();
    // tactical_operation_parameters.insert(1, operation_parameter);

    // let tactical_work_order_parameter =
    //     TacticalParameter::new(work_order,
    // tactical_operation_parameters);

    // tactical_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, tactical_work_order_parameter);

    // let activity_number = 0;

    // let mut tactical_activities = TacticalScheduledOperations::default();

    // tactical_activities.0.insert(
    //     activity_number,
    //     OperationSolution::new(
    //         vec![],
    //         Resources::MtnMech,
    //         1,
    //         Work::from(0.0),
    //         work_order_number,
    //         activity_number,
    //     ),
    // );

    // tactical_algorithm
    //     .solution
    //     .tactical_scheduled_work_orders
    //     .0
    //     .insert(
    //         work_order_number,
    //         WhereIsWorkOrder::Tactical(tactical_activities),
    //     );

    // tactical_algorithm.schedule().unwrap();

    // let scheduled_date = tactical_algorithm
    //     .solution
    //     .tactical_scheduled_days(&work_order_number, 0);

    // assert!(scheduled_date.is_ok());
}

#[test]
fn test_schedule_2()
{
    let work_order_number = WorkOrderNumber(2100000010);
    let activity_number = 1;
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let tactical_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };

    let id = Id::default();
    let options = TacticalOptions::default();
    let scheduling_environment = SchedulingEnvironment::default();

    // SchedulingEnvironment

    let tactical_parameters = TacticalParameters::new(&id, options, &scheduling_environment)?;
    let tactical_solution = TacticalSolution::new(&tactical_parameters);

    let mut tactical_algorithm = Algorithm::new(
        tactical_days(56),
        TacticalResources::new_from_data(
            Resources::iter().collect(),
            tactical_days(56),
            Work::from(100.0),
        ),
        TacticalResources::new_from_data(
            Resources::iter().collect(),
            tactical_days(56),
            Work::from(0.0),
        ),
        ArcSwapSharedSolution::default().into(),
    );

    let mut tactical_activities = TacticalScheduledOperations::default();

    tactical_activities.0.insert(
        activity_number,
        OperationSolution::new(
            vec![],
            Resources::MtnMech,
            1,
            Work::from(0.0),
            work_order_number,
            activity_number,
        ),
    );

    tactical_algorithm
        .solution
        .tactical_scheduled_work_orders
        .0
        .insert(
            work_order_number,
            WhereIsWorkOrder::Tactical(tactical_activities),
        );

    // Operation
    // 1,
    // Work::from(1.0),
    // Work::from(1.0),
    // Work::from(1.0),
    // Resources::MtnMech,
    let operation_parameter = OperationParameter::new(work_order_number, operation);

    let mut operation_parameters = HashMap::new();
    operation_parameters.insert(1, operation_parameter);

    // Work Order
    // Resources::MtnMech,
    // 10,
    // vec![],
    // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
    let optimized_tactical_work_order = TacticalParameter::new(work_order, operation_parameters);

    tactical_algorithm
        .parameters_mut()
        .insert(work_order_number, optimized_tactical_work_order);

    tactical_algorithm.schedule().unwrap();

    let scheduled_date = tactical_algorithm
        .solution
        .tactical_scheduled_days(&work_order_number, 1);

    assert!(scheduled_date.is_ok());
}
