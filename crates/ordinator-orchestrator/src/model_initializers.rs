use std::sync::Arc;
use std::sync::Mutex;

use arc_swap::Guard;
use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::IntoSchedulingEnvironment;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_total_data_processing::sources::baptiste_csv_reader::TotalSap;

pub fn initialize_scheduling_environment(
    system_configurations: Guard<Arc<SystemConfigurations>>,
) -> Arc<Mutex<SchedulingEnvironment>>
{
    let total_sap = TotalSap::default();

    // FIX [ ]
    // This is completely wrong! I think that we should create it in a completely
    // different way to make this system work. You should not define the builder
    // in the `SchedulingEnvironment` itself, but rely on the
    // `ordinator-orchestrator` crate or the `ordinator-total-data-processing`
    // crate. QUESTION [ ]
    // How should you structure this to make it work in a correct way?
    TotalSap::into_scheduling_environment(total_sap, &system_configurations)
        .expect("Could not load the data from the data file")
}
