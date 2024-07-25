use std::env;
use std::path::Path;

use tracing::info;

use crate::data_processing::sources::{excel::TotalExcel, SchedulingEnvironmentFactory};
use shared_messages::scheduling_environment::SchedulingEnvironment;

pub fn initialize_scheduling_environment(
    number_of_strategic_periods: u32,
    number_of_tactical_periods: u32,
    number_of_days: u32,
) -> SchedulingEnvironment {
    let mut scheduling_environment = create_scheduling_environment(
        number_of_strategic_periods,
        number_of_tactical_periods,
        number_of_days,
    )
    .expect("No data file was provided.");
    scheduling_environment
        .initialize_work_orders(&scheduling_environment.clone_strategic_periods());
    scheduling_environment.initialize_worker_environment();
    info!("{}", scheduling_environment);
    scheduling_environment
}

fn create_scheduling_environment(
    number_of_strategic_periods: u32,
    number_of_tactical_periods: u32,
    number_of_days: u32,
) -> Option<SchedulingEnvironment> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let file_path = Path::new(&args[1]);

        let total_excel = TotalExcel::new(
            file_path,
            number_of_strategic_periods,
            number_of_tactical_periods,
            number_of_days,
        );

        let scheduling_environment =
            SchedulingEnvironment::create_scheduling_environment(total_excel)
                .unwrap_or_else(|_| panic!("Could not load data file. File path: {:?} ", args));
        return Some(scheduling_environment);
    }
    None
}
