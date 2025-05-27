use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::SchedulingEnvironment;

use super::model_initializers;

pub struct DataBaseConnection {}

// At the moment you are simply reading everything into a
// single struct and then you forget about the MongoDB
// connection. I do not think that is a good approach
// here. You should be able to interact with the data
// continuously for this to work.
impl DataBaseConnection
{
    // ISSUE #000 move-temp-scheduling-environment-into-mongodb
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    {
        Self {}
    }

    pub fn scheduling_environment(
        system_configuration: Arc<ArcSwap<SystemConfigurations>>,
    ) -> Result<Arc<Mutex<SchedulingEnvironment>>>
    {
        let database_path = &system_configuration.load().database_config;
        if database_path.exists() {
            initialize_from_database(database_path)
        } else {
            initialize_from_source_data_and_initialize_database(system_configuration.load())
                .context("Could not write SchedulingEnvironment to database.")
        }
    }
}

fn initialize_from_database(path: &Path) -> Result<Arc<Mutex<SchedulingEnvironment>>>
{
    let mut file = File::open(path)?;
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(Arc::new(Mutex::new(serde_json::from_str::<
        SchedulingEnvironment,
    >(&data)?)))
}

fn initialize_from_source_data_and_initialize_database(
    system_configurations: arc_swap::Guard<Arc<SystemConfigurations>>,
) -> Result<Arc<Mutex<SchedulingEnvironment>>>
{
    let file_path = system_configurations.database_config.clone();
    let scheduling_environment =
        model_initializers::initialize_scheduling_environment(system_configurations)
            .context("Could not initialize the SchedulingEnvironment from source data")?;

    let json_scheduling_environment =
        serde_json::to_string(&*scheduling_environment.lock().unwrap()).unwrap();
    // TODO [ ]
    // Make database integration here.
    let mut file = File::create(file_path).unwrap();

    file.write_all(json_scheduling_environment.as_bytes())
        .unwrap();
    Ok(scheduling_environment)
}
