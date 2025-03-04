use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use config::Config;
use data_processing::sources::TimeInput;
use rand::{rngs::StdRng, SeedableRng};
use shared_types::{
    configuration::{
        material::MaterialToPeriod, throttling::Throttling, toml_baptiste::BaptisteToml,
        user_interface::EventColors,
    },
    scheduling_environment::work_order::WorkOrderConfigurations,
    ActorSpecifications, Asset,
};

use crate::agents::{
    operational_agent::OperationalOptions, strategic_agent::StrategicOptions,
    supervisor_agent::SupervisorOptions, tactical_agent::TacticalOptions,
};

/// This struct is used to load in all configuraions centrally into the Orchestrator.
/// The `Orchestrator` then uses dependency injection to provide the actors with the
/// correct `Configurations`.
///
// There is something that you do not understand here. Where should
// all these configurations go?
// WARN
// Remember! You have a single source of all configurations here,
// so there is no reason to question that in the system.
pub struct SystemConfigurations {
    work_order_configurations: WorkOrderConfigurations,
    actor_configurations: ActorConfigurations,
    actor_specification: HashMap<Asset, ActorSpecifications>,
    pub data_locations: BaptisteToml,
    throttling: Throttling,
    // FIX
    // This should be more general later on.
    user_interface: EventColors,
    // TODO [ ]
    // Extend this for mongodb if ever needed.
    pub database_config: PathBuf,
    pub time_input: TimeInput,
    pub material_to_period: MaterialToPeriod,
}

// Okay the `Option`s are looking okay, the options
// are related to the functioning of the `Actor`s and
// the `Configuration`s are related to how the data in
// the `SchedulingEnvironment` is intrepreted by the
// actors, this is a completely different concern and
//
// should be handled as such. Good! Good progress.
// TODO [ ]
// We should remove the `Default` on all `Option`s
// and then move a file for each of them
struct ActorConfigurations {
    strategic_options: StrategicOptionsConfig,
    tactical_options: TacticalOptionsConfig,
    supervisor_options: SupervisorOptionsConfig,
    operational_options: OperationalOptionsConfig,
}

// FIX [ ]
// This is a good initial approach but remember to make it better if you have to revisit it.
impl SystemConfigurations {
    pub fn read_all_configs() -> Result<SystemConfigurations> {
        let work_order_configurations: WorkOrderConfigurations =
            serde_json::from_str("./configuration/work_orders/work_order_weight_parameters.json")?;

        let list_of_actor_specification = vec![
            (
                Asset::Df,
                "./configuration/actor_specification/actor_specification_df.toml",
            ),
            (
                Asset::Hb,
                "./configuration/actor_specification/actor_specification_hb.toml",
            ),
            (
                Asset::Hd,
                "./configuration/actor_specification/actor_specification_hd.toml",
            ),
            (
                Asset::Test,
                "./configuration/actor_specification/actor_specification_test.toml",
            ),
            (
                Asset::Te,
                "./configuration/actor_specification/actor_specification_te.toml",
            ),
        ];

        let actor_specification: HashMap<Asset, ActorSpecifications> = list_of_actor_specification
            .iter()
            .map(|(asset, path)| {
                let contents = std::fs::read_to_string(path).unwrap();
                let config: ActorSpecifications = toml::from_str(&contents).unwrap();
                (asset, config)
            })
            .collect();

        let operational_options_content =
            std::fs::read_to_string("./configuration/actor_options/operational_options.toml")
                .unwrap();
        let strategic_options_content =
            std::fs::read_to_string("./configuration/actor_options/strategic_options.toml")
                .unwrap();
        let supervisor_options_content =
            std::fs::read_to_string("./configuration/actor_options/supervisor_options.toml")
                .unwrap();
        let tactical_options_content =
            std::fs::read_to_string("./configuration/actor_options/tactical_options.toml").unwrap();
        let operational_options = toml::from_str(&operational_options_content).unwarp();
        let strategic_options = toml::from_str(&strategic_options_content).unwarp();
        let supervisor_options = toml::from_str(&supervisor_options_content).unwarp();
        let tactical_options = toml::from_str(&tactical_options_content).unwarp();

        let actor_configurations: ActorConfigurations = ActorConfigurations {
            operational_options,
            strategic_options,
            supervisor_options,
            tactical_options,
        };

        let baptiste_data_locations_contents =
            std::fs::read_to_string("./configuration/data_locations/baptiste_data_locations.toml")
                .unwrap();
        let data_locations = toml::from_str(&baptiste_data_locations_contents).unwrap();

        let throttling_contents =
            std::fs::read_to_string("./configuration/throttling/throttling.toml").unwrap();
        let throttling = toml::from_str(&throttling_contents).unwrap();

        let event_colors_contents =
            std::fs::read_to_string("./configuration/user_interface/event_colors.toml").unwrap();
        let event_colors = toml::from_str(&event_colors_contents).unwrap();

        let database_path_string =
            &dotenvy::var("DATABASE_PATH").expect("Could not read database path");

        let database_path = std::path::Path::new(database_path_string);

        let time_input_contents =
            std::fs::read_to_string("./configuration/time_environment/time_inputs.toml").unwrap();
        let time_input = toml::from_str(&time_input_contents).unwrap();

        let material_to_period_contents =
            std::fs::read_to_string("./configuration/materials/status_to_period.toml").unwrap();
        let material_to_period: MaterialToPeriod =
            toml::from_str(&material_to_period_contents).unwrap();

        Ok(SystemConfigurations {
            work_order_configurations,
            actor_configurations,
            actor_specification,
            data_locations,
            throttling,
            user_interface: event_colors,
            database_config: database_path.to_owned(),
            time_input,
            material_to_period,
        })
        // TODO [ ]
        // Integrate this if you have issues with data initialization
        // let file_string = dotenvy::var("ORDINATOR_INPUT")
        //     .expect("The ORDINATOR_INPUT environment variable have to be set");

        // let mut file_path = PathBuf::new();

        // file_path.push(&file_string);
    }

    pub fn strategic_options(&self) -> StrategicOptions {
        let number_of_removed_work_order = self
            .actor_configurations
            .strategic_options
            .number_of_removed_work_orders;
        let urgency_weight = self.actor_configurations.strategic_options.urgency_weight;
        let resource_penalty_weight = self
            .actor_configurations
            .strategic_options
            .resource_penalty_weight;
        let clustering_weight = self
            .actor_configurations
            .strategic_options
            .clustering_weight;
        let work_order_configurations = self.work_order_configurations;

        let material_to_period = self.material_to_period;

        let rng = StdRng::from_os_rng();
        // QUESTION [ ]
        // _Should this field be private or public?_
        //
        // You should provide an ID here to solve this problem.
        StrategicOptions {
            number_of_removed_work_order,
            rng,
            urgency_weight,
            resource_penalty_weight,
            clustering_weight,
            work_order_configurations,
            material_to_period,
        }
    }
    pub fn tactical_options(&self) -> TacticalOptions {
        TacticalOptions {
            number_of_removed_work_orders: self
                .actor_configurations
                .tactical_options
                .number_of_removed_work_orders,
            rng: StdRng::from_os_rng(),
        }
    }

    pub fn supervisor_options(&self) -> SupervisorOptions {
        let number_of_unassigned_work_orders = self
            .actor_configurations
            .supervisor_options
            .number_of_removed_work_orders;
        SupervisorOptions {
            rng: StdRng::from_os_rng(),
            number_of_unassigned_work_orders,
        }
    }

    pub fn operational_options(&self) -> OperationalOptions {
        let number_of_removed_activities = self
            .actor_configurations
            .operational_options
            .number_of_removed_work_orders;
        OperationalOptions {
            rng: StdRng::from_os_rng(),
            number_of_removed_activities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SystemConfigurations;

    #[test]
    fn test_read_config() {
        let system_configurations = SystemConfigurations::read_all_configs().unwrap();

        println!("{:#?}", system_configurations);
    }
}
