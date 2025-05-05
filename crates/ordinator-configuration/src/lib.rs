mod material;
mod resources;
mod throttling;
pub mod time_input;
pub mod toml_baptiste;
mod user_interface;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_scheduling_environment::SystemConfigurationTrait;
use throttling::Throttling;
use toml_baptiste::BaptisteToml;
use user_interface::EventColors;

// QUESTION
// How should this be handled?
// They should be handled by created by handling a `From<<Actor>OptionConfig>
// for <Actor>Config` in the `ordinator-actors` crate!

/// This struct is used to load in all configuraions centrally into the
/// Orchestrator. The `Orchestrator` then uses dependency injection to provide
/// the actors with the correct `Configurations`.
// There is something that you do not understand here. Where should
// all these configurations go?
// WARN
// Remember! You have a single source of all configurations here,
// so there is no reason to question that in the system.
#[derive(Debug)]
pub struct SystemConfigurations {
    pub data_locations: BaptisteToml,
    pub throttling: Throttling,
    pub user_interface: EventColors,
    pub database_config: PathBuf,
}

impl SystemConfigurationTrait for SystemConfigurations {}

// FIX [ ]
// This is a good initial approach but remember to make it better if you have to
// revisit it.
impl SystemConfigurations {
    pub fn read_all_configs() -> Result<Arc<ArcSwap<SystemConfigurations>>> {
        let baptiste_data_locations_contents =
            std::fs::read_to_string("./configuration/data_locations/baptiste_data_locations.toml")
                .unwrap();
        let data_locations = toml::from_str(&baptiste_data_locations_contents).unwrap();

        let throttling_contents =
            std::fs::read_to_string("./configuration/throttling/throttling.toml").unwrap();
        let throttling: Throttling = toml::from_str(&throttling_contents).unwrap();

        let event_colors_contents =
            std::fs::read_to_string("./configuration/user_interface/event_colors.toml").unwrap();
        let event_colors: EventColors = toml::from_str(&event_colors_contents).unwrap();

        let database_path_string =
            &dotenvy::var("DATABASE_PATH").expect("Could not read database path");

        let database_path = std::path::Path::new(database_path_string);

        // I believe that it is the best appraoch here to make sure that the
        // `Configurations` are always created wrapped. Then you will never
        // make the mistake, of accessing wild and stray configurations.
        Ok(Arc::new(ArcSwap::new(Arc::new(SystemConfigurations {
            data_locations,
            throttling,
            user_interface: event_colors,
            database_config: database_path.to_owned(),
        }))))
        // TODO [ ]
        // Integrate this if you have issues with data initialization
        // let file_string = dotenvy::var("ORDINATOR_INPUT")
        //     .expect("The ORDINATOR_INPUT environment variable have to be
        // set");

        // let mut file_path = PathBuf::new();

        // file_path.push(&file_string);
    }

    // This is actually a `From <SystemConfiguration> for StrateticOptions`
}
// This should be a part of the creation of the `SchedulingEnvironment`
// #[test]
// fn test_read_config() {
//     let system_configurations = SystemConfigurations::read_all_configs().unwrap();

//     println!("{:#?}", system_configurations);
// }
