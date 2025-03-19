use data_processing::sources::SchedulingEnvironmentFactory;
use data_processing::sources::TimeInput;
use data_processing::sources::baptiste_csv_reader::TotalSap;
use shared_types::scheduling_environment::SchedulingEnvironment;
use tracing::info;

use super::configuration::SystemConfigurations;

pub fn initialize_scheduling_environment(
    system_configurations: SystemConfigurations,
) -> SchedulingEnvironment {
    let total_sap = TotalSap::new(system_configurations.data_locations);

    // FIX [ ]
    // This is completely wrong! I think that we should create it in a completely
    // different way to make this system work. You should not define the builder
    // in the `SchedulingEnvironment` itself, but rely on the
    // `ordinator-orchestrator` crate or the `ordinator-total-data-processing`
    // crate. QUESTION [ ]
    // How should you structure this to make it work in a correct way?
    SchedulingEnvironment::create_scheduling_environment(
        total_sap,
        system_configurations.time_input,
    )
    .expect("Could not load the data from the data file")
}
