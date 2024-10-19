use std::path::PathBuf;

use data_processing::sources::TimeInput;
use tracing::info;

use data_processing::sources::{baptiste_csv_reader::TotalSap, SchedulingEnvironmentFactory};
use shared_types::scheduling_environment::SchedulingEnvironment;

pub fn initialize_scheduling_environment(
    number_of_strategic_periods: u64,
    number_of_tactical_periods: u64,
    number_of_days: u64,
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
    number_of_strategic_periods: u64,
    number_of_tactical_periods: u64,
    number_of_days: u64,
) -> Option<SchedulingEnvironment> {
    let file_string = dotenvy::var("ORDINATOR_INPUT")
        .expect("The ORDINATOR_INPUT environment variable have to be set");

    let mut file_path = PathBuf::new();

    file_path.push(&file_string);

    let time_input = TimeInput::new(
        number_of_strategic_periods,
        number_of_tactical_periods,
        number_of_days,
    );

    let total_sap = TotalSap::new(file_path);

    let scheduling_environment =
        SchedulingEnvironment::create_scheduling_environment(total_sap, time_input)
            .expect("Could not load the data from the data file");
    return Some(scheduling_environment);
}
