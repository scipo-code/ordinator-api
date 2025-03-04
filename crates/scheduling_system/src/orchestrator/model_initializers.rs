use data_processing::sources::TimeInput;
use tracing::info;

use data_processing::sources::{baptiste_csv_reader::TotalSap, SchedulingEnvironmentFactory};
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::configuration::SystemConfigurations;

pub fn initialize_scheduling_environment(
    system_configurations: SystemConfigurations,
) -> SchedulingEnvironment {
    let total_sap = TotalSap::new(system_configurations.data_locations);

    let scheduling_environment = SchedulingEnvironment::create_scheduling_environment(
        total_sap,
        system_configurations.time_input,
    )
    .expect("Could not load the data from the data file");

    info!("{}", scheduling_environment);
    scheduling_environment
}
