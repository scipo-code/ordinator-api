use shared_types::{
    scheduling_environment::work_order::WorkOrderConfigurations, ActorSpecifications,
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
struct SystemConfigurations {
    work_order_configurations: WorkOrderConfigurations,
    actor_configurations: ActorConfigurations,
    actor_specification: ActorSpecifications,
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
    strategic_options: StrategicOptions,
    tactical_options: TacticalOptions,
    supervisor_options: SupervisorOptions,
    operational_options: OperationalOptions,
}


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
