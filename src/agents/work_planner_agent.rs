pub struct WorkPlannerAgent {
    id: i32,
    orders: Vec<u32>,
}



// fn call_julia_matheuristic(scheduling_environment) -> JLrsResult {

//     let julia = unsafe { Julia::new() };

//     julia.eval_string("using Scheduling")?;

//     let scheduling_environment = load_data_file(file_path).expect("Could not load data file.");

//     let result = julia.call::<SchedulingEnvironment, _>("Scheduling.Large_Neighborhood_Search", (scheduling_environment,))?;

//     Ok(result)
// }