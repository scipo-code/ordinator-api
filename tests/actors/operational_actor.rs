#[test]
fn test_determine_first_available_start_time() -> Result<()> {
    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();
    let options = OperationalOptions::from(SystemConfigurations::read_all_configs().unwrap());

    let operational_algorithm: OperationalAlgorithm<SharedSolution> = Algorithm::builder()
        .id(id)
        .parameters(&options, &scheduling_environment)?
        .solution()
        .build();

    operational_algorithm.load_shared_solution();

    let mut strategic_updated_shared_solution =
        (**operational_algorithm.loaded_shared_solution).clone();

    // Here you can simply access what it is that you need? Is that not a better
    // approach? No I do not think that it is. I believe that we are doing it the
    // You have a hard time making these test after what you have done here. I
    // you lose the ability to simply insert things as you like into the
    // other actors and that could hamper you ability to test edge cases...
    // No that is actually a good thing.
    strategic_updated_shared_solution
        .strategic
        .strategic_scheduled_work_orders
        .insert(
            WorkOrderNumber(0),
            Some(Period::from_str("2024-W41-42").unwrap()),
        );

    operational_algorithm
        .arc_swap_shared_solution
        .0
        .store(Arc::new(strategic_updated_shared_solution));

    operational_algorithm.load_shared_solution();
    let mut tactical_updated_shared_solution =
        (**operational_algorithm.loaded_shared_solution).clone();

    tactical_updated_shared_solution
        .tactical
        .tactical_work_orders
        .0
        .insert(WorkOrderNumber(0), WhereIsWorkOrder::NotScheduled);

    operational_algorithm
        .arc_swap_shared_solution
        .0
        .store(Arc::new(tactical_updated_shared_solution));

    operational_algorithm.load_shared_solution();

    let operational_parameter = OperationalParameter::new(Work::from(20.0), Work::from(0.0))
        .expect("Work has to be non-zero to create an OperationalParameter");

    let start_time = operational_algorithm
        .determine_first_available_start_time(&(WorkOrderNumber(0), 0), &operational_parameter)
        .unwrap();

    assert_eq!(
        start_time,
        DateTime::parse_from_rfc3339("2024-10-07T08:00:00Z")
            .unwrap()
            .to_utc()
    );
    Ok(())
}
#[test]
fn test_determine_next_event_3() -> Result<()> {
    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();

    let value = SystemConfigurations::read_all_configs().unwrap();
    let options = OperationalOptions::from(value);

    let operational_algorithm: OperationalAlgorithm<SharedSolution> = Algorithm::builder()
        .id(id)
        .parameters(&options, &scheduling_environment)?
        .solution()
        .build();

    let current_time = DateTime::parse_from_rfc3339("2024-05-20T01:00:00Z")
        .unwrap()
        .to_utc();

    let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

    assert_eq!(time_delta, TimeDelta::new(3600 * 6, 0).unwrap());
    // assert_eq!(next_event,
    // OperationalEvents::Toolbox(value.actor_configurations.));
    Ok(())
}
#[test]
fn test_determine_next_event_2() -> Result<()> {
    // let mut scheduling_environment = SchedulingEnvironment::default();

    // let id = &Id::new("TEST_OPERATIONAL", vec![], vec![]);

    // // Here we would require a method to create an OperationalActor instead.
    // // Having the data available is also a good idea. I think that injecting
    // // time as a dependency is really the most important architectual thing here
    // // for making all of this work. It will also allow us to live in the past and
    // // test on old data. You need to work like this if you want to make this work.
    // let operational_configuration_all =
    //     OperationalConfigurationAll::new(id.clone(), 6.0, operational_configuration);

    // scheduling_environment
    //     .worker_environment
    //     .agent_environment
    //     .operational
    //     .insert(id.clone(), operational_configuration_all);

    // let scheduling_environment = Arc::new(Mutex::new(scheduling_environment));

    // let options = SystemConfigurations::read_all_configs().unwrap();

    // let operational_algorithm = Algorithm::builder()
    //     .id(id)
    //     .parameters(
    //         OperationalOptions::from::<SystemConfigurations>(options),
    //         scheduling_environment,
    //     )?
    //     .build();

    // let current_time = DateTime::parse_from_rfc3339("2024-05-20T00:00:00Z")
    //     .unwrap()
    //     .to_utc();

    // let (time_delta, next_event) =
    // FIX This should be a function on data. NOT a method. You do not need the
    // required context of the whole algorithm to determine the timing of the
    // next event.
    // operational_algorithm.determine_next_event(&current_time);
    // FIX

    // assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());
    // assert_eq!(next_event, OperationalEvents::Toolbox(toolbox_interval));
    Ok(())
}
#[test]
fn test_determine_next_event_1() -> Result<()> {
    let system_configurations = SystemConfigurations::read_all_configs()?;

    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    // This is a huge no go. Operational agents should be initialize from the
    // configs
    // FIX [ ]
    // This should be initialized from a central place
    scheduling_environment
        .worker_environment
        .agent_environment
        .operational
        .insert(id.clone(), operational_configuration_all);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();

    // What should be created or changed here? I think that the best appraoch is to
    // make a trait for each builder.

    // This means that it is not better to use the system_configurations.
    // You have to test using something else. I think that here the best option
    // is to use the
    // FIX You derived the system configurations here for testing purposes. This
    // is not the fastest way to approach this.
    // let operational_options = OperationalOptions::from(system_configurations);

    // Algorithms need to be generic over the `SharedSolution` I do not see another
    // way of doing it.

    let operational_algorithm: OperationalAlgorithm<Ss> = Algorithm::builder()
        .id(id)
        // Parameters not does not need the `options`
        .parameters(&scheduling_environment)?
        .solution()
        .build();

    let current_time = DateTime::parse_from_rfc3339("2024-05-20T12:00:00Z")
        .unwrap()
        .to_utc();

    let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

    assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());

    assert_eq!(next_event, OperationalEvents::OffShift(off_shift_interval));
    Ok(())
}
