use std::path::PathBuf;

use data_processing::sources::TimeInput;
use tracing::info;

use data_processing::sources::{baptiste_csv_reader::TotalSap, SchedulingEnvironmentFactory};
use shared_types::scheduling_environment::SchedulingEnvironment;

pub fn initialize_scheduling_environment(time_input: TimeInput, input_config: ) -> SchedulingEnvironment {
    let file_string = dotenvy::var("ORDINATOR_INPUT")
        .expect("The ORDINATOR_INPUT environment variable have to be set");

    let mut file_path = PathBuf::new();

    file_path.push(&file_string);

    let total_sap = TotalSap::new(file_path);

    let mut scheduling_environment =
        SchedulingEnvironment::create_scheduling_environment(total_sap, time_input)
            .expect("Could not load the data from the data file");

    scheduling_environment.initialize_work_orders(
        &scheduling_environment
            .time_environment
            .strategic_periods
            .clone(),
    );
    // scheduling_environment
    //     .worker_environment
    //     .initialize_worker_environment();

    info!("{}", scheduling_environment);
    scheduling_environment
}
