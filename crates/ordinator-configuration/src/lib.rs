mod actor_specifications;
mod material;
mod resources;
mod throttling;
pub mod time_input;
pub mod toml_baptiste;
mod user_interface;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use actor_specifications::ActorSpecifications;
use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SystemConfigurationTrait;
use ordinator_scheduling_environment::time_environment::MaterialToPeriod;
use ordinator_scheduling_environment::work_order::WorkOrderConfigurations;
use throttling::Throttling;
use time_input::TimeInput;
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
    pub work_order_configurations: WorkOrderConfigurations,
    // This should be derived for the single actor only. Not all of them.
    // That is important.
    // So this should be moved. Where should it be moved to? I think that this
    //
    // TODO [ ] Move it
    pub data_locations: BaptisteToml,
    pub throttling: Throttling,
    // FIX
    // This should be more general later on.
    pub user_interface: EventColors,
    // TODO [ ]
    // Extend this for mongodb if ever needed.
    pub database_config: PathBuf,
    pub time_input: TimeInput,
    pub material_to_period: MaterialToPeriod,
}

impl SystemConfigurationTrait for SystemConfigurations {}

// FIX [ ]
// This is a good initial approach but remember to make it better if you have to
// revisit it.
impl SystemConfigurations {
    pub fn read_all_configs() -> Result<Arc<ArcSwap<SystemConfigurations>>> {
        let work_order_configurations: WorkOrderConfigurations =
            serde_json::from_str("./configuration/work_orders/work_order_weight_parameters.json")?;

        let list_of_actor_specification = vec![
            (
                Asset::DF,
                "./configuration/actor_specification/actor_specification_df.toml",
            ),
            (
                Asset::HB,
                "./configuration/actor_specification/actor_specification_hb.toml",
            ),
            (
                Asset::HD,
                "./configuration/actor_specification/actor_specification_hd.toml",
            ),
            (
                Asset::Test,
                "./configuration/actor_specification/actor_specification_test.toml",
            ),
            (
                Asset::TE,
                "./configuration/actor_specification/actor_specification_te.toml",
            ),
        ];

        let actor_specification: HashMap<Asset, ActorSpecifications> = list_of_actor_specification
            .into_iter()
            .map(|(asset, path)| {
                let contents = std::fs::read_to_string(path).unwrap();
                let config: ActorSpecifications = toml::from_str(&contents).unwrap();
                (asset, config)
            })
            .collect();

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

        let time_input_contents =
            std::fs::read_to_string("./configuration/time_environment/time_inputs.toml").unwrap();
        let time_input: TimeInput = toml::from_str(&time_input_contents).unwrap();

        let material_to_period_contents =
            std::fs::read_to_string("./configuration/materials/status_to_period.toml").unwrap();
        let material_to_period: MaterialToPeriod =
            toml::from_str(&material_to_period_contents).unwrap();

        // I believe that it is the best appraoch here to make sure that the
        // `Configurations` are always created wrapped. Then you will never
        // make the mistake, of accessing wild and stray configurations.
        Ok(Arc::new(ArcSwap::new(Arc::new(SystemConfigurations {
            work_order_configurations,
            actor_specification,
            data_locations,
            throttling,
            user_interface: event_colors,
            database_config: database_path.to_owned(),
            time_input,
            material_to_period,
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

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;
    use ordinator_scheduling_environment::worker_environment::resources::Resources;

    use super::SystemConfigurations;
    use crate::ActorSpecifications;

    #[test]
    fn test_read_config() {
        let system_configurations = SystemConfigurations::read_all_configs().unwrap();

        println!("{:#?}", system_configurations);
    }

    #[test]
    fn test_toml_operational_parsing() {
        let toml_operational_string = r#"
            [[supervisors]]
            id = "main"
            number_of_supervisAgentEnvironmentr_periods = 3

            # [[supervisors]]
            # id = "supervisor-second"
            ################################
            ###          MTN-ELEC        ###
            ################################
            [[operational]]
            id = "OP-01-001"
            resources.resources = ["MTN-ELEC" ]
            hours_per_day = 6.0
            operational_configuration.off_shift_interval = { start = "19:00:00",  end = "07:00:00" }
            operational_configuration.break_interval = { start = "11:00:00", end = "12:00:00" }
            operational_configuration.toolbox_interval = { start = "07:00:00", end = "08:00:00" }
            operational_configuration.availability.start_date = "2024-12-02T07:00:00Z"
            operational_configuration.availability.finish_date = "2024-12-15T15:00:00Z"
        "#;

        let system_agents: ActorSpecifications = toml::from_str(toml_operational_string).unwrap();

        assert_eq!(system_agents.operational[0].id, "OP-01-001".to_string());

        assert_eq!(system_agents.operational[0].resources, [Resources::MtnElec]);

        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .start,
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        );
        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .end,
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
    }
}
