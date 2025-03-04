use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use shared_types::scheduling_environment::SchedulingEnvironment;

use super::{configuration::SystemConfigurations, model_initializers};

pub struct DataBaseConnection {}

// At the moment you are simply reading everything into a
// single struct and then you forget about the MongoDB
// connection. I do not think that is a good approach
// here. You should be able to interact with the data
// continuously for this to work.
impl DataBaseConnection {
    pub fn new() -> Self {
        Self {}
    }
    pub fn scheduling_environment(
        system_configuration: SystemConfigurations,
    ) -> Arc<Mutex<SchedulingEnvironment>> {
        let database_path = &system_configuration.database_config;
        let scheduling_environment = if database_path.exists() {
            initialize_from_database(database_path)
        } else {
            initialize_from_source_data_and_initialize_database(database_path)
                .expect("Could not write SchedulingEnvironment to database.")
        };

        Arc::new(Mutex::new(scheduling_environment))
    }
}

fn initialize_from_database(path: &Path) -> SchedulingEnvironment {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    serde_json::from_str::<SchedulingEnvironment>(&data).unwrap()
}

fn initialize_from_source_data_and_initialize_database(
    system_configurations: SystemConfigurations,
) -> Result<SchedulingEnvironment, std::io::Error> {
    // FIX [ ]
    // You should use the whole configuration when initializing the `SchedulingEnvironment`
    // QUESTION
    // What is the best way of keeping all this consistent?
    //

    let scheduling_environment =
        model_initializers::initialize_scheduling_environment(system_configurations);

    let json_scheduling_environment = serde_json::to_string(&scheduling_environment).unwrap();
    let mut file = File::create(path).unwrap();

    file.write_all(json_scheduling_environment.as_bytes())
        .unwrap();
    Ok(scheduling_environment)
}
