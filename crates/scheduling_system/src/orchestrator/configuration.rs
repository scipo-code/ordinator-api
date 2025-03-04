use config::Config;
use shared_types::{
    configuration::{throttling::Throttling, toml_baptiste::BaptisteToml}, scheduling_environment::work_order::WorkOrderConfigurations, ActorSpecifications, Asset
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
    data_locations: BaptisteToml,
    throttling: Throttling,
    // FIX
    // This should be more general later on.
    user_interface: EventColors,
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

struct StrategicOptionConfig {
    number_of_removed_work_orders: usize,
    urgency_weight: usize,
    resource_penalty_weight: usize,
    clustering_weight: usize,
}


impl SystemConfigurations {
    pub fn read_all_configs() -> Result<()> {

        let work_order_config: WorkOrderConfigurations = serde_json::from_str("./configuration/work_orders/work_order_weight_parameters.json")?;

        let list_of_actor_specification = vec![
           (Asset::Df, "./configuration/actor_specification/actor_specification_df.toml"),       
           (Asset::Hb, "./configuration/actor_specification/actor_specification_hb.toml"),       
           (Asset::Hd, "./configuration/actor_specification/actor_specification_hd.toml"),       
           (Asset::Test, "./configuration/actor_specification/actor_specification_test.toml"),       
           (Asset::Te, "./configuration/actor_specification/actor_specification_te.toml"),       
        ];

        let actor_specification: HashMap<Asset, ActorSpecifications> = list_of_actor_specification.iter().map(|(asset, path)| {
            let contents = std::fs::read_to_string(path).unwrap();
            let config: ActorSpecifications = toml::from_str(&contents).unwrap();
            (asset, config)
        }).collect();

        let operational_options_content = std::fs::read_to_string("./configuration/actor_options/operational_options.toml").unwrap();
        let strategic_options_content = std::fs::read_to_string("./configuration/actor_options/strategic_options.toml").unwrap();
        let supervisor_options_content = std::fs::read_to_string("./configuration/actor_options/supervisor_options.toml").unwrap();
        let tactical_options_content = std::fs::read_to_string("./configuration/actor_options/tactical_options.toml").unwrap();
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

        let baptiste_data_locations_contents = std::fs::read_to_string("./configuration/data_locations/baptiste_data_locations.toml").unwrap();
        let data_locations =  toml::from_str(&baptiste_data_locations_contents).unwrap();

        let throttling_contents = std::fs::read_to_string("./configuration/throttling/throttling.toml").unwrap();
        let throttling =  toml::from_str(&throttling_contents).unwrap();

        let event_colors_contents = std::fs::read_to_string("./configuration/user_interface/event_colors.toml").unwrap();
        let event_colors = toml::from_str(&event_colors_contents).unwrap();

        Ok(SystemConfigurations {
                    work_order_configurations,
                    actor_configurations,
                    actor_specification,
                    data_locations,
                    throttling,
                    user_interface,
                })
    }
}


// You have more that you need to move around here.
// FIX [ ]
// Configuration is laying in a nested random place in the code. This should be centralized so that the whole
// program can be changed from a CSV Dump to a Mongodb connection.
//
// TODO [ ]
// Take a break to think about how to structure the code. 
// Operating time should also be derived, I think. 

#[derive(Deserialize, Debug)]
pub struct TomlOperatingTime {
    operating_time: f64,
}

let toml_operating_time_string =
    fs::read_to_string("./configuration/operating_time.toml").unwrap();
let operating_time: TomlOperatingTime = toml::from_str(&toml_operating_time_string).unwrap();

// TODO [ ] 
// Move this into the central configuration
let clustering_weights: ClusteringWeights = {
    let clustering_config_path = dotenvy::var("CLUSTER_WEIGHTINGS")
        .context("CLUSTER_WEIGHTINGS should be defined in the env")?;
    let clustering_config_contents = std::fs::read_to_string(clustering_config_path)
        .context("Could not read config file")?;
    serde_json::from_str(&clustering_config_contents)?
};
pub fn initialize_from_resource_configuration_file(
    &mut self,
    system_agents_bytes: Vec<u8>,
) -> Result<()> {
    // This is a complete mess! How could you create it like this! 
    let contents = std::str::from_utf8(&system_agents_bytes)
        .context("configuration file bitstream not read correct")?;

    let system_agents: ActorSpecifications = toml::from_str(contents)
        .with_context(|| format!("configuration file string could not be parsed into {}. Likely a toml parsing error", std::any::type_name::<ActorSpecifications>().bright_red()))?;

    self.agent_environment = system_agents.into();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SystemConfigurations;



    #[test]
    fn test_read_config() {
        SystemConfigurations::read_all_configs()
    }

    #[test]    
    fn test_read_all_configs() -> Result<()> {


        let settings = Config::builder()
            .add_source(config::File::with_name("./configuration/work_orders/work_order_weight_parameters.json"))
            .add_source(config::File::with_name("./configuration/actor_specification/actor_specification_df.toml"))
            .add_source(config::File::with_name("./configuration/actor_specification/actor_specification_hb.toml"))
            .add_source(config::File::with_name("./configuration/actor_specification/actor_specification_hd.toml"))
            .add_source(config::File::with_name("./configuration/actor_specification/actor_specification_test.toml"))
            .add_source(config::File::with_name("./configuration/actor_specification/actor_specification_te.toml"))
            .add_source(config::File::with_name("./configuration/actor_options/operational_options.toml"))
            .add_source(config::File::with_name("./configuration/actor_options/strategic_options.toml"))
            .add_source(config::File::with_name("./configuration/actor_options/supervisor_options.toml"))
            .add_source(config::File::with_name("./configuration/actor_options/tactical_options.toml"))
            .add_source(config::File::with_name("./configuration/data_locations/baptiste_data_locations.toml"))
            .add_source(config::File::with_name("./configuration/throttling/throttling.toml"))
            .add_source(config::File::with_name("./configuration/user_interface/event_colors.toml"))
            .build()?;


        println!("{:#?}", settings);

        
        Ok(())
    }
    
}
